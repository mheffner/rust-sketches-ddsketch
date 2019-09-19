const DEFAULT_MAX_BINS: u32 = 2048;
const DEFAULT_ALPHA: f64 = 0.01;
const DEFAULT_MIN_VALUE: f64 = 1.0e-9;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Config {
    pub max_num_bins: u32,
    pub gamma: f64,
    gamma_ln: f64,
    min_value: f64,
    pub offset: i32
}

fn log_gamma(value: f64, gamma_ln: f64) -> f64
{
    value.ln() / gamma_ln
}

impl Config {
    pub fn new(alpha: f64, max_num_bins: u32, min_value: f64) -> Self {
        let gamma_ln = (2.0 * alpha) / (1.0 - alpha);
        let gamma_ln = gamma_ln.ln_1p();

        Config{
            max_num_bins: max_num_bins,
            gamma: 1.0 + (2.0 * alpha) / (1.0 - alpha),
            gamma_ln,
            min_value: min_value,
            offset: 1 - (log_gamma(min_value, gamma_ln) as i32)
        }
    }

    pub fn defaults() -> Self {
        Self::new(DEFAULT_ALPHA, DEFAULT_MAX_BINS, DEFAULT_MIN_VALUE)
    }

    pub fn key(self: &Self, v: f64) -> i32 {
        if v < -self.min_value {
            return -(self.log_gamma(-v).ceil() as i32) - self.offset;
        } else if v > self.min_value {
            return (self.log_gamma(v).ceil() as i32) + self.offset;
        } else {
            return 0;
        }
    }

    pub fn log_gamma(self: &Self, value: f64) -> f64 {
        log_gamma(value, self.gamma_ln)
    }

    pub fn pow_gamma(self: &Self, k: i32) -> f64 {
        ((k as f64) * self.gamma_ln).exp()
    }
}