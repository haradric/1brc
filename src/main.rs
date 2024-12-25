use std::collections::{BTreeMap, HashMap};

use itertools::Itertools;

const FILENAME: &str = "measurements_1b.txt";

fn main() {
    let mut measurements = HashMap::new();

    std::fs::read_to_string(FILENAME)
        .expect("Failed to read the file")
        .lines()
        .for_each(|line| {
            let mut parts = line.split_terminator(';');
            let station = parts.next().expect("Invalid line format");
            let temperature = parts
                .next()
                .unwrap()
                .parse::<f64>()
                .expect("Invalid value format");

            measurements
                .entry(station.to_owned())
                .or_insert_with(Vec::new)
                .push(temperature);
        });

    let results = measurements
        .into_iter()
        .map(|(station, temps)| {
            let min = *temps
                .iter()
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap();
            let avg = temps.iter().sum::<f64>() / temps.len() as f64;
            let max = *temps
                .iter()
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap();

            (station, (min, avg, max))
        })
        .collect::<BTreeMap<String, (f64, f64, f64)>>();

    print!("{{");
    #[allow(unstable_name_collisions)]
    results
        .iter()
        .map(|(station, (min, avg, max))| format!("{station}={min:.1}/{avg:.1}/{max:.1}"))
        .intersperse(", ".to_string())
        .for_each(|s| print!("{s}"));
    println!("}}");
}
