use std::cmp::{max, min};
use std::io::copy;
use std::ops::{RangeFrom};
use std::fmt;

const GROW_LEFT_BY: i32 = 128;

fn new_vec(size: usize) -> Vec<i64> {
    vec![0; size]
}

// TODO: cleanup int types here
pub struct Store {
    bins: Vec<i64>,
    count: i64,
    min_key: i32,
    max_key: i32,
    max_num_bins: i32
}

impl Store {
    pub fn new(max_num_bins: i32) -> Self {
        Store {
            bins: new_vec(max_num_bins as usize),
            count: 0,
            min_key: 0,
            max_key: 0,
            max_num_bins: max_num_bins
        }
    }

    pub fn length(self: &Self) -> i32 {
        self.bins.len() as i32
    }

    pub fn add(self: &mut Self, key: i32) {
        if self.count == 0 {
            self.max_key = key;
            self.min_key = key - (self.length() as i32) + 1;
        }
        if key < self.min_key {
            self.grow_left(key)
        } else if key > self.max_key {
            self.grow_right(key)
        }

        let idx = max(key - self.min_key, 0) as usize;

        self.bins[idx] += 1;
        self.count += 1;
    }

    pub fn key_at_rank(self: &Self, rank: i32) -> i32 {
        let mut n = 0i32;
        for (i, bin) in self.bins.iter().enumerate() {
            // XXX: overflow?
            n += *bin as i32;
            if n >= rank {
                return i as i32 + self.min_key;
            }
        }

        self.max_key
    }

    fn grow_left(self: &mut Self, key: i32) {
        if self.min_key < key || self.length() >= self.max_num_bins {
            return
        }

        let mut min_key;
        if self.max_key - key >= self.max_num_bins as i32 {
            min_key = self.max_key - (self.max_num_bins as i32) + 1
        } else {
            min_key = -self.min_key;

            // XXX: remove this loop
            while min_key > key {
                min_key -= GROW_LEFT_BY;
            }
        }


        let mut tmp_bins = new_vec((self.max_key - min_key + 1) as usize);
        //let mut tmp_bins = Vec::with_capacity((self.max_key - min_key + 1) as usize);
        //copy(tmp_bins[self.min_key - min_key..], self.bins);
        let tmp_slice = &mut tmp_bins[self.convert_range((self.min_key - min_key)..)];
        tmp_slice.copy_from_slice(&self.bins[0..]);
        self.bins = tmp_bins;
        self.min_key = min_key;
    }

    fn grow_right(self: &mut Self, key: i32) {
        if self.max_key > key {
            return
        }
        if key - self.max_key >= self.max_num_bins {
            self.bins = new_vec(self.max_num_bins as usize);
            self.max_key = key;
            self.min_key = key - self.max_num_bins + 1;
            self.bins[0] = self.count
        } else if key - self.min_key >= self.max_num_bins {
            let min_key = key - self.max_num_bins + 1;
            let mut n = 0;
            for i in self.min_key..min(self.min_key, self.max_key + 1) {
                n += self.bins[(i - self.min_key) as usize];
            }
            if self.length() < self.max_num_bins {
                let mut tmp_bins = new_vec(self.max_num_bins as usize);
                let tmp_slice = &mut tmp_bins;
                tmp_slice.copy_from_slice(&self.bins[self.convert_range((min_key - self.min_key)..)]);
                self.bins = tmp_bins;
            } else {
                self.bins.drain(0..((min_key - self.min_key) as usize));
                /*
                {
                    println!("bins len: {}, min_key: {}, self.min_key: {}, mid: {}", self.bins.len(), min_key, self.min_key, min_key - self.min_key);
                    println!("self.max_key: {}, copy from loc: {}", self.max_key, (self.max_key - min_key + 1));
                    let (left, right) = self.bins.split_at_mut((min_key - self.min_key) as usize);
                    left.copy_from_slice(&right[0..]);
                }
                */
                for i in (self.max_key - min_key + 1)..self.max_num_bins {
                    //self.bins[i as usize] = 0
                    self.bins.push(0);
                }
            }
            self.max_key = key;
            self.min_key = min_key;
            self.bins[0] += n;
        } else {
            let mut tmp_bins = new_vec((key - self.min_key + 1) as usize);
            let tmp_slice = &mut tmp_bins;
            tmp_slice.copy_from_slice(&self.bins);
            self.bins = tmp_bins;
            self.max_key = key;
        }
    }

    fn convert_range(self: &Self, range: RangeFrom<i32>) -> RangeFrom<usize> {
        assert!(range.start >= 0);
        RangeFrom { start: range.start as usize }
    }
}

impl fmt::Debug for Store {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Store {{ bins: ")?;
        for i in 0..self.bins.len() {
            write!(f, "{}: {}, ", (i as i32) + self.min_key, self.bins[i])?;
        }
        writeln!(f, "count: {}, min_key: {}, max_key: {} }}", self.count, self.min_key, self.max_key)
    }
}

#[cfg(test)]
mod tests {
    use crate::ddsketch::store::Store;

    #[test]
    fn test_simple_store()
    {
        let mut s = Store::new(2048);

        for i in 0..2048 {
            println!("Adding: {}", i);
            s.add(i);
        }
        println!("Store: {:?}", s);
    }
}