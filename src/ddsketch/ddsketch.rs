use std::error;
use std::fmt;
use std::f64::INFINITY;

use super::config::Config;
use super::store::Store;

type Result<T> = std::result::Result<T, QuantileError>;

#[derive(Debug, Clone)]
pub struct QuantileError;

impl fmt::Display for QuantileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid quantile, must be between 0 and 1 inclusive")
    }
}
impl error::Error for QuantileError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        // Generic
        None
    }
}

pub struct DDSketch {
    config: Config,
    store: Store,
    min: f64,
    max: f64,
    count: u64, // usize?
    sum: f64
}

// XXX: functions should return Option<> in the case of empty
impl DDSketch {
    pub fn new(config: Config) -> Self {
        DDSketch {
            config: config,
            store: Store::new(config.max_num_bins as i32),
            min: INFINITY,
            max: -INFINITY,
            count: 0,
            sum: 0.0
        }
    }

    pub fn add(self: &mut Self, v: f64) {
        let key = self.config.key(v);

        self.store.add(key);

        if v < self.min {
            self.min = v;
        }
        if self.max < v {
            self.max = v;
        }
        self.count += 1;
        self.sum += v;
    }

    pub fn quantile(self: &Self, q: f64) -> Result<Option<f64>> {
        if q < 0.0 || q > 1.0 {
            return Err(QuantileError)
        }

        if self.empty() {
            return Ok(None)
        }

        if q == 0.0 {
            return Ok(Some(self.min));
        } else if q == 1.0 {
            return Ok(Some(self.max));
        }

        let rank = (q * ((self.count - 1) as f64) + 1.0) as i32;
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

    pub fn min(self: &Self) -> Option<f64> {
        if self.empty() {
            None
        } else {
            Some(self.min)
        }
    }

    pub fn max(self: &Self) -> Option<f64> {
        if self.empty() {
            None
        } else {
            Some(self.max)
        }
    }

    pub fn sum(self: &Self) -> Option<f64> {
        if self.empty() {
            None
        } else {
            Some(self.sum)
        }
    }

    pub fn count(self: &Self) -> usize {
        self.count as usize
    }

    fn empty(self: &Self) -> bool {
        self.count == 0
    }
}

#[cfg(test)]
mod tests {
    use crate::ddsketch::config::Config;
    use crate::ddsketch::ddsketch::DDSketch;

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


}