use std::collections::{BTreeMap, HashMap};

use itertools::Itertools;

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

    fn finalize(self) -> (f32, f32, f32) {
        #[allow(clippy::cast_precision_loss)]
        let avg = self.sum / self.count as f32;
        (self.min, avg, self.max)
    }
}

fn main() {
    let mut measurements = HashMap::new();

    std::fs::read_to_string(FILENAME)
        .expect("Failed to read the file")
        .lines()
        .for_each(|line| {
            let (station, value) = line.split_once(';').expect("Invalid line format");
            let value = value.parse::<f32>().expect("Invalid value format");

            measurements
                .entry(station.to_owned())
                .and_modify(|entry: &mut Stats| {
                    entry.add(value);
                })
                .or_insert_with(|| {
                    let mut entry = Stats::new();
                    entry.add(value);
                    entry
                });
        });

    let results = measurements
        .into_iter()
        .map(|(station, stats)| (station, stats.finalize()))
        .collect::<BTreeMap<String, (f32, f32, f32)>>();

    print!("{{");
    #[allow(unstable_name_collisions)]
    results
        .iter()
        .map(|(station, (min, avg, max))| format!("{station}={min:.1}/{avg:.1}/{max:.1}"))
        .intersperse(", ".to_string())
        .for_each(|s| print!("{s}"));
    println!("}}");
}
