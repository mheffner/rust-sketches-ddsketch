use crate::ddsketch::config::Config;
use std::f64::{INFINITY, NAN};
use crate::ddsketch::store::Store;

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

    pub fn quantile(self: &Self, q: f64) -> f64 {
        if q < 0.0 || q > 1.0 || self.count == 0 {
            return NAN;
        }

        if q == 0.0 {
            return self.min;
        } else if q == 1.0 {
            return self.max;
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
        if quantile < self.min {
            return self.min;
        } else if quantile > self.max {
            return self.max;
        } else {
            return quantile;
        }
    }

    pub fn min(self: &Self) -> f64 {
        self.min
    }

    pub fn max(self: &Self) -> f64 {
        self.max
    }

    pub fn sum(self: &Self) -> f64 {
        self.sum
    }

    pub fn count(self: &Self) -> usize {
        self.count as usize
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

        assert_eq!(dd.quantile(0.95).ceil(), 95.0);
    }


}