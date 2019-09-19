use sketches_ddsketch::ddsketch::config::Config;
use sketches_ddsketch::ddsketch::ddsketch::DDSketch;

mod common;
use common::dataset;
use common::generator;
use common::generator::Generator;
use std::time::Instant;

const TEST_ALPHA: f64 = 0.01;
const TEST_MAX_BINS: u32 = 1024;
const TEST_MIN_VALUE: f64 = 1.0e-9;

const TEST_SIZES: [usize; 5] = [3, 5, 10, 100, 1000];
const TEST_QUANTILES: [f64; 10] = [0.0, 0.1, 0.25, 0.5, 0.75, 0.9, 0.95, 0.99, 0.999, 1.0];

#[test]
fn test_constant() {
    evaluate_sketches(|| Box::new(generator::Constant::new(42.0)));
}

#[test]
fn test_linear() {
    evaluate_sketches(|| Box::new(generator::Linear::new(0.0, 1.0)));
}

#[test]
fn test_normal() {
    evaluate_sketches(|| Box::new(generator::Normal::new(35.0, 1.0)));
}

#[test]
fn test_lognormal() {
    evaluate_sketches(|| Box::new(generator::Lognormal::new(0.0, 2.0)));
}

#[test]
fn test_exponential() {
    evaluate_sketches(|| Box::new(generator::Exponential::new(2.0)));
}

fn evaluate_sketches(gen_factory: impl Fn() -> Box<dyn generator::Generator>) {
    for sz in &TEST_SIZES {
        let mut generator = gen_factory();
        evaluate_sketch(*sz, &mut generator);
    }

}

fn evaluate_sketch(count: usize, generator: &mut Box<dyn generator::Generator>) {
    let c = Config::new(TEST_ALPHA, TEST_MAX_BINS, TEST_MIN_VALUE);
    let mut g = DDSketch::new(c);

    let mut d = dataset::Dataset::new();

    for _i in 0..count {
        let value = generator.generate();

        g.add(value);
        d.add(value);
    }

    compare_sketches(&mut d, &g);
}

fn compare_sketches(d: &mut dataset::Dataset, g: &DDSketch) {
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

        let quantile = g.quantile(*q).unwrap().unwrap();

        assert!(min_expected <= quantile, "Lower than min, quantile: {}, wanted {} <= {}", *q, min_expected, quantile);
        assert!(quantile <= max_expected, "Higher than max, quantile: {}, wanted {} <= {}", *q, quantile, max_expected);

        // verify that calls do not modify result (not mut so not possible?)
        let quantile2 = g.quantile(*q).unwrap().unwrap();
        assert_eq!(quantile, quantile2);
    }

    assert_eq!(g.min().unwrap(), d.min());
    assert_eq!(g.max().unwrap(), d.max());
    assert_eq!(g.sum().unwrap(), d.sum());
    assert_eq!(g.count(), d.count());
}

// Uncomment to run, preferably only in release mode
//#[test]
fn test_performance() {
    let c = Config::defaults();
    let mut g = DDSketch::new(c);
    let mut gen = generator::Normal::new(1000.0, 500.0);
    let count = 300_000_000;

    let mut values = Vec::new();
    for _ in 0..count {
        values.push(gen.generate());
    }

    let start_time = Instant::now();
    for value in values {
        g.add(value);
    }
    let quantile = g.quantile(0.50).unwrap().unwrap();

    let elapsed = start_time.elapsed().as_micros() as f64;
    let elapsed = elapsed / 1_000_000.0;

    println!("p50={:.2} => Added {}M samples in {:2} secs ({:.2}M samples/sec)", quantile,
             count / 1_000_000, elapsed, (count as f64) / 1_000_000.0 / elapsed);
}