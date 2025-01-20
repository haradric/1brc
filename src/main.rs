#![feature(slice_split_once)]

use std::collections::HashMap;
use std::io::Read;

const FILENAME: &str = "measurements_1b.txt";

struct Stats {
    min: f32,
    max: f32,
    sum: f32,
    count: usize,
}

impl Stats {
    const fn new() -> Self {
        Self {
            min: f32::INFINITY,
            max: f32::NEG_INFINITY,
            sum: 0.0,
            count: 0,
        }
    }

    fn add(&mut self, value: f32) {
        self.min = self.min.min(value);
        self.max = self.max.max(value);
        self.sum += value;
        self.count += 1;
    }

    #[allow(clippy::cast_precision_loss)]
    fn avg(&self) -> f32 {
        self.sum / self.count as f32
    }
}

fn main() {
    let mut data = vec![];
    std::fs::File::open(FILENAME)
        .expect("Failed to open the file")
        .read_to_end(&mut data)
        .expect("Failed to read the file");
    assert!(data.pop() == Some(b'\n'));

    let mut measurements = HashMap::new();
    for line in data.split(|&c| c == b'\n') {
        let (station, value) = line
            .split_once(|&c| c == b';')
            .expect("Invalid line format");

        let value = unsafe { std::str::from_utf8_unchecked(value) }
            .parse::<f32>()
            .expect("Invalid value format");

        measurements
            .entry(station)
            .or_insert(Stats::new())
            .add(value);
    }

    let mut results = measurements.into_iter().collect::<Vec<_>>();
    results.sort_unstable_by_key(|entry| entry.0);

    print!("{{");
    for (station, stats) in &results {
        print!(
            "{:}={:.1}/{:.1}/{:.1}, ",
            std::str::from_utf8(station).expect("Invalid station name"),
            stats.min,
            stats.avg(),
            stats.max
        );
    }
    println!("}}");
}
