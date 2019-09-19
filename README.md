# sketches-ddsketch

This is a direct port of the [Golang](https://github.com/DataDog/sketches-go) 
[DDSketch](https://arxiv.org/pdf/1908.10693.pdf) quantile sketch implementation 
to Rust.

# DDSketch

## Usage

```rust
let config = Config::defaults();
let mut sketch = DDSketch::new(c);

sketch.add(1.0);
sketch.add(2.0);
sketch.add(3.0);

// Get p=50%
let quantile: Option<f64> = sketch.quantile(0.5).unwrap(); 
```

## Performance

No performance tuning has been done with this implementation of the port, so we
would expect similar profiles to the original implementation.

Out of the box we see can achieve over 70M sample inserts/sec and 350K sketch
merges/sec. All tests run on a single core Intel i7 processor with 4.2Ghz max 
clock.