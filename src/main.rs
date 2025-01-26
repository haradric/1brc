use std::collections::HashMap;

use memchr::memchr;

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
        assert!((-999..=999).contains(&value));
        self.min = self.min.min(value);
        self.max = self.max.max(value);
        self.sum += value;
        self.count += 1;
    }

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_possible_wrap)]
    const fn avg(&self) -> i32 {
        assert!(self.count > 0);
        self.sum / self.count as i32
    }

    fn merge(&mut self, other: &Self) {
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
        self.sum += other.sum;
        self.count += other.count;
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

fn read_file(path: &str) -> Vec<u8> {
    use std::io::Read;

    let mut buf = vec![];

    std::fs::File::open(path)
        .expect("Failed to open the file")
        .read_to_end(&mut buf)
        .expect("Failed to read the file");

    buf
}

fn process(mut data: &[u8]) -> HashMap<u64, (&[u8], Stats)> {
    let mut stats = HashMap::new();

    loop {
        let Some(station_length) = memchr(b';', data) else {
            break;
        };

        // 1 for the separator
        let value_length = memchr(b'\n', &data[station_length + 1..]).expect("Invalid format");
        let station = &data[..station_length];
        let value = &data[station_length + 1..station_length + 1 + value_length];

        stats
            .entry(str_to_key(station))
            .or_insert((station, Stats::new()))
            .1
            .add(parse_to_fixed_point(value));

        // 1 for the separator and 1 for the newline
        data = &data[station_length + 1 + value_length + 1..];
    }

    stats
}

fn process_parallel(data: &[u8], jobs: usize) -> HashMap<u64, (&[u8], Stats)> {
    let mut stats_merged = HashMap::new();

    std::thread::scope(|s| {
        let step = data.len() / jobs;

        let mut handles = Vec::with_capacity(jobs);
        let mut first = 0;

        while first + step < data.len() {
            let last = first + step;

            let pos = memchr(b'\n', &data[last..]).expect("File must end with newline");
            handles.push(s.spawn(move || process(&data[first..=last + pos])));

            first = last + pos + 1;
        }
        handles.push(s.spawn(move || process(&data[first..data.len()])));

        handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .for_each(|item| {
                for (key, (station, stats)) in item {
                    stats_merged
                        .entry(key)
                        .and_modify(|e: &mut (&[u8], Stats)| e.1.merge(&stats))
                        .or_insert((station, stats));
                }
            });
    });

    stats_merged
}

fn main() {
    let buffer = read_file(FILENAME);

    let jobs = std::thread::available_parallelism().unwrap().get();
    let stats = process_parallel(&buffer, jobs);

    let mut results = stats.into_iter().collect::<Vec<_>>();
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
    use super::*;

    #[test]
    fn stats_add() {
        let mut stats = Stats::new();

        stats.add(123);
        assert_eq!(stats.min, 123);
        assert_eq!(stats.max, 123);
        assert_eq!(stats.sum, 123);
        assert_eq!(stats.count, 1);

        stats.add(321);
        assert_eq!(stats.min, 123);
        assert_eq!(stats.max, 321);
        assert_eq!(stats.sum, 444);
        assert_eq!(stats.count, 2);

        stats.add(-456);
        assert_eq!(stats.min, -456);
        assert_eq!(stats.max, 321);
        assert_eq!(stats.sum, -12);
        assert_eq!(stats.count, 3);
    }

    #[test]
    fn stats_avg() {
        let mut stats = Stats::new();
        for i in [123, 321, -456] {
            stats.add(i);
        }
        assert_eq!(stats.avg(), -4);
    }

    #[test]
    fn stats_merge() {
        let mut stats1 = Stats::new();
        for i in [123, 321, -456] {
            stats1.add(i);
        }

        let mut stats2 = Stats::new();
        for i in [456, -678, -987] {
            stats2.add(i);
        }

        stats1.merge(&stats2);
        assert_eq!(stats1.min, -987);
        assert_eq!(stats1.max, 456);
        assert_eq!(stats1.sum, -1221);
        assert_eq!(stats1.count, 6);
    }

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
