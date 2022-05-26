use std::error;
use std::f64::INFINITY;
use std::fmt;

use crate::config::Config;
use crate::store::Store;

type Result<T> = std::result::Result<T, DDSketchError>;

/// General error type for DDSketch, represents either an invalid quantile or an
/// incompatible merge operation.
///
#[derive(Debug, Clone)]
pub enum DDSketchError {
    Quantile,
    Merge,
}
impl fmt::Display for DDSketchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DDSketchError::Quantile => {
                write!(f, "Invalid quantile, must be between 0 and 1 (inclusive)")
            }
            DDSketchError::Merge => write!(f, "Can not merge sketches with different configs"),
        }
    }
}
impl error::Error for DDSketchError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        // Generic
        None
    }
}

/// This struct represents a [DDSketch](https://arxiv.org/pdf/1908.10693.pdf)
#[derive(Clone)]
pub struct DDSketch {
    config: Config,
    store: Store,
    min: f64,
    max: f64,
    sum: f64,
}

// XXX: functions should return Option<> in the case of empty
impl DDSketch {
    /// Construct a `DDSketch`. Requires a `Config` specifying the parameters of the sketch
    pub fn new(config: Config) -> Self {
        DDSketch {
            config: config,
            store: Store::new(config.max_num_bins as i32),
            min: INFINITY,
            max: -INFINITY,
            sum: 0.0,
        }
    }

    /// Add the sample to the sketch
    pub fn add(&mut self, v: f64) {
        let key = self.config.key(v);

        self.store.add(key);

        if v < self.min {
            self.min = v;
        }
        if self.max < v {
            self.max = v;
        }
        self.sum += v;
    }

    /// Return the quantile value for quantiles between 0.0 and 1.0. Result is an error, represented
    /// as DDSketchError::Quantile if the requested quantile is outside of that range.
    ///
    /// If the sketch is empty the result is None, else Some(v) for the quantile value.
    pub fn quantile(&self, q: f64) -> Result<Option<f64>> {
        if q < 0.0 || q > 1.0 {
            return Err(DDSketchError::Quantile);
        }

        if self.empty() {
            return Ok(None);
        }

        if q == 0.0 {
            return Ok(Some(self.min));
        } else if q == 1.0 {
            return Ok(Some(self.max));
        }

        let rank = (q * self.count() as f64).ceil() as u64;
        let mut key = self.store.key_at_rank(rank);

        let quantile;
        if key < 0 {
            key += self.config.offset;
            quantile = -2.0 * self.config.pow_gamma(-key) / (1.0 + self.config.gamma);
        } else if key > 0 {
            key -= self.config.offset;
            quantile = 2.0 * self.config.pow_gamma(key) / (1.0 + self.config.gamma);
        } else {
            quantile = 0.0;
        }

        // Bound by the extremes
        let ret;
        if quantile < self.min {
            ret = self.min;
        } else if quantile > self.max {
            ret = self.max;
        } else {
            ret = quantile;
        }

        Ok(Some(ret))
    }

    /// Returns the minimum value seen, or None if sketch is empty
    pub fn min(&self) -> Option<f64> {
        if self.empty() {
            None
        } else {
            Some(self.min)
        }
    }

    /// Returns the maximum value seen, or None if sketch is empty
    pub fn max(&self) -> Option<f64> {
        if self.empty() {
            None
        } else {
            Some(self.max)
        }
    }

    /// Returns the sum of values seen, or None if sketch is empty
    pub fn sum(&self) -> Option<f64> {
        if self.empty() {
            None
        } else {
            Some(self.sum)
        }
    }

    /// Returns the number of values added to the sketch
    pub fn count(&self) -> usize {
        self.store.count() as usize
    }

    /// Returns the length of the underlying `Store`. This is mainly only useful for understanding
    /// how much the sketch has grown given the inserted values.
    pub fn length(&self) -> usize {
        self.store.length() as usize
    }

    /// Merge the contents of another sketch into this one. The sketch that is merged into this one
    /// is unchanged after the merge.
    pub fn merge(&mut self, o: &DDSketch) -> Result<()> {
        if self.config != o.config {
            return Err(DDSketchError::Merge);
        }

        let was_empty = self.store.count() == 0;

        // Merge the stores
        self.store.merge(&o.store);

        // Need to ensure we don't override min/max with initializers
        // if either store were empty
        if was_empty {
            self.min = o.min;
            self.max = o.max;
        } else if o.store.count() > 0 {
            if o.min < self.min {
                self.min = o.min
            }
            if o.max > self.max {
                self.max = o.max;
            }
        }
        self.sum += o.sum;

        Ok(())
    }

    fn empty(&self) -> bool {
        self.count() == 0
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use crate::Config;
    use crate::DDSketch;

    #[test]
    fn test_quartiles() {
        let alpha = 0.01;
        let c = Config::new(alpha, 2048, 10e-9);
        let mut dd = DDSketch::new(c);

        // Initialize sketch with {1.0, 2.0, 3.0, 4.0}
        for i in 1..5 {
            dd.add(i as f64);
        }

        // We expect the following mappings from quantile to value:
        // [0,0.25]: 1.0, (0.25,0.5]: 2.0, (0.5,0.75]: 3.0, (0.75, 1.0]: 4.0
        let test_cases = vec![
            (0.0, 1.0),
            (0.25, 1.0),
            (0.26, 2.0),
            (0.5, 2.0),
            (0.51, 3.0),
            (0.75, 3.0),
            (0.76, 4.0),
            (1.0, 4.0),
        ];

        for (q, val) in test_cases {
            assert_relative_eq!(dd.quantile(q).unwrap().unwrap(), val, max_relative = alpha);
        }
    }

    #[test]
    fn test_simple_quantile() {
        let c = Config::defaults();
        let mut dd = DDSketch::new(c);

        for i in 1..101 {
            dd.add(i as f64);
        }

        assert_eq!(dd.quantile(0.95).unwrap().unwrap().ceil(), 95.0);

        assert!(dd.quantile(-1.01).is_err());
        assert!(dd.quantile(1.01).is_err());
    }

    #[test]
    fn test_empty_sketch() {
        let c = Config::defaults();
        let dd = DDSketch::new(c);

        assert_eq!(dd.quantile(0.98).unwrap(), None);
        assert_eq!(dd.max(), None);
        assert_eq!(dd.min(), None);
        assert_eq!(dd.sum(), None);
        assert_eq!(dd.count(), 0);

        assert!(dd.quantile(1.01).is_err());
    }

    #[test]
    fn test_basic_histogram_data() {
        let values = &[
            0.754225035,
            0.752900282,
            0.752812246,
            0.752602367,
            0.754310155,
            0.753525981,
            0.752981082,
            0.752715536,
            0.751667941,
            0.755079054,
            0.753528150,
            0.755188464,
            0.752508723,
            0.750064549,
            0.753960428,
            0.751139298,
            0.752523560,
            0.753253428,
            0.753498342,
            0.751858358,
            0.752104636,
            0.753841300,
            0.754467374,
            0.753814334,
            0.750881719,
            0.753182556,
            0.752576884,
            0.753945708,
            0.753571911,
            0.752314573,
            0.752586651,
        ];

        let c = Config::defaults();
        let mut dd = DDSketch::new(c);

        for value in values {
            dd.add(*value);
        }

        assert_eq!(dd.max(), Some(0.755188464));
        assert_eq!(dd.min(), Some(0.750064549));
        assert_eq!(dd.count(), 31);
        assert_eq!(dd.sum(), Some(23.343630625000003));

        assert!(dd.quantile(0.25).unwrap().is_some());
        assert!(dd.quantile(0.5).unwrap().is_some());
        assert!(dd.quantile(0.75).unwrap().is_some());
    }
}
