use sketches_ddsketch::ddsketch::config::Config;
use sketches_ddsketch::ddsketch::ddsketch::DDSketch;
use crate::common::dataset::Dataset;
use std::cmp::max;

mod common;

const TEST_ALPHA: f64 = 0.01;
const TEST_QUANTILES: [f64; 10] = [0.0, 0.1, 0.25, 0.5, 0.75, 0.9, 0.95, 0.99, 0.999, 1.0];

#[test]
fn test_constant_gen() {
    let c = Config::defaults();
    let mut g = DDSketch::new(c);

    let mut d = common::dataset::Dataset::new();

    for i in 0..200 {
        let value = 42.0f64;

        g.add(value);
        d.add(value);
    }

    compare_sketches(&mut d, &g);
    println!("Looking for 95% == {}", d.quantile(0.95));
}

fn compare_sketches(d: &mut Dataset, g: &DDSketch) {
    for q in &TEST_QUANTILES {
        let lower = d.lower_quantile(*q);
        let upper = d.upper_quantile(*q);

        let min_expected;
        if lower < 0.0 {
            min_expected = lower * (1.0 + TEST_ALPHA);
        } else {
            min_expected = lower * (1.0 - TEST_ALPHA);
        }

        let max_expected;
        if upper > 0.0 {
            max_expected = upper * (1.0 + TEST_ALPHA);
        } else {
            max_expected = upper * (1.0 - TEST_ALPHA);
        }

        let quantile = g.quantile(*q);

        assert!(min_expected <= quantile, "Quantile: {}", *q);
        assert!(quantile <= max_expected, "Quantile: {}", *q);
    }

    assert_eq!(g.min(), d.min());
    assert_eq!(g.max(), d.max());
    assert_eq!(g.sum(), d.sum());
    assert_eq!(g.count(), d.count());
}