#![feature(slice_split_once)]

use std::io::Read;

const FILENAME: &str = "measurements_1b.txt";

// values stored as fixed point numbers
struct Stats {
    min: i32,
    max: i32,
    sum: i32,
    count: usize,
}

impl Stats {
    const fn new() -> Self {
        Self {
            min: i32::MAX,
            max: i32::MIN,
            sum: 0,
            count: 0,
        }
    }

    fn add(&mut self, value: i32) {
        self.min = self.min.min(value);
        self.max = self.max.max(value);
        self.sum += value;
        self.count += 1;
    }

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_possible_wrap)]
    const fn avg(&self) -> i32 {
        self.sum / self.count as i32
    }
}

#[allow(clippy::cast_possible_truncation)]
fn str_to_key(name: &[u8]) -> u64 {
    let mut key = [0u8; 8];

    let length = name.len().min(8);

    key[..length].copy_from_slice(&name[..length]);
    key[0] ^= name.len() as u8;

    u64::from_ne_bytes(key)
}

fn parse_to_fixed_point(mut s: &[u8]) -> i32 {
    let sign = if s.starts_with(b"-") {
        s = &s[1..];
        -1
    } else {
        1
    };

    // 98.7 -> 987
    // 10 * a + b + 0.1 * c -> 100 * a + 10 * b + c
    let (a, b, c) = match s {
        [a, b, b'.', c] => (
            i32::from(*a) - i32::from(b'0'),
            i32::from(*b) - i32::from(b'0'),
            i32::from(*c) - i32::from(b'0'),
        ),
        [b, b'.', c] => (
            0i32,
            i32::from(*b) - i32::from(b'0'),
            i32::from(*c) - i32::from(b'0'),
        ),
        _ => panic!("Invalid format for {s:?}"),
    };

    sign * (100 * a + 10 * b + c)
}

fn fixed_to_float(value: i32) -> f64 {
    f64::from(value) / 10.0
}

fn main() {
    let mut data = vec![];
    std::fs::File::open(FILENAME)
        .expect("Failed to open the file")
        .read_to_end(&mut data)
        .expect("Failed to read the file");
    assert!(data.pop() == Some(b'\n'));

    let mut measurements = std::collections::HashMap::new();
    for line in data.split(|&c| c == b'\n') {
        let (station, value) = line
            .split_once(|&c| c == b';')
            .expect("Invalid line format");

        measurements
            .entry(str_to_key(station))
            .or_insert((station, Stats::new()))
            .1
            .add(parse_to_fixed_point(value));
    }

    let mut results = measurements.into_iter().collect::<Vec<_>>();
    results.sort_unstable_by_key(|(_key, value)| value.0);

    print!("{{");
    for (_, (station, stats)) in &results {
        print!(
            "{:}={:.1}/{:.1}/{:.1}, ",
            std::str::from_utf8(station).expect("Invalid station name"),
            fixed_to_float(stats.min),
            fixed_to_float(stats.avg()),
            fixed_to_float(stats.max)
        );
    }
    println!("}}");
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse_to_fixed_point() {
        use super::parse_to_fixed_point;

        assert_eq!(parse_to_fixed_point(b"98.7"), 987);
        assert_eq!(parse_to_fixed_point(b"10.0"), 100);
        assert_eq!(parse_to_fixed_point(b"9.8"), 98);
        assert_eq!(parse_to_fixed_point(b"0.9"), 9);
        assert_eq!(parse_to_fixed_point(b"0.1"), 1);
        assert_eq!(parse_to_fixed_point(b"0.0"), 0);
        assert_eq!(parse_to_fixed_point(b"-0.1"), -1);
        assert_eq!(parse_to_fixed_point(b"-0.9"), -9);
        assert_eq!(parse_to_fixed_point(b"-9.8"), -98);
        assert_eq!(parse_to_fixed_point(b"-10.0"), -100);
        assert_eq!(parse_to_fixed_point(b"-98.7"), -987);
    }
}
