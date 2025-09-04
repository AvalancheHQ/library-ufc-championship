use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
struct TestData {
    id: u64,
    name: String,
    values: Vec<i32>,
    metadata: Option<String>,
}

impl TestData {
    fn sample() -> Self {
        Self {
            id: 12345,
            name: "test_user".to_string(),
            values: (1..=10_000).collect(), // Much larger vector: 1000 elements instead of 8
            metadata: Some("sample metadata with much more content to simulate larger payloads in real world scenarios where we need to serialize and deserialize substantial amounts of data".to_string()),
        }
    }
}

fn main() {
    divan::main();
}

#[divan::bench_group(min_time = Duration::from_secs(3))]
mod serialize {
    use super::*;

    #[divan::bench]
    fn serde_json(bencher: divan::Bencher) {
        let data = TestData::sample();

        bencher
            .with_inputs(|| data.clone())
            .bench_values(|data| serde_json::to_string(&data).unwrap())
    }

    #[divan::bench]
    fn sonic_rs(bencher: divan::Bencher) {
        let data = TestData::sample();

        bencher
            .with_inputs(|| data.clone())
            .bench_values(|data| sonic_rs::to_string(&data).unwrap())
    }
}

#[divan::bench_group(min_time = Duration::from_secs(3))]
mod deserialize {
    use super::*;

    #[divan::bench]
    fn serde_json(bencher: divan::Bencher) {
        let data = TestData::sample();
        let json = serde_json::to_string(&data).unwrap();

        bencher.with_inputs(|| json.clone()).bench_values(|json| {
            let _: TestData = serde_json::from_str(&json).unwrap();
        })
    }

    #[divan::bench]
    fn sonic_rs(bencher: divan::Bencher) {
        let data = TestData::sample();
        let json = sonic_rs::to_string(&data).unwrap();

        bencher.with_inputs(|| json.clone()).bench_values(|json| {
            let _: TestData = sonic_rs::from_str(&json).unwrap();
        })
    }
}

#[divan::bench_group(min_time = Duration::from_secs(3))]
mod round_trip {
    use super::*;

    #[divan::bench]
    fn serde_json(bencher: divan::Bencher) {
        let data = TestData::sample();

        bencher.with_inputs(|| data.clone()).bench_values(|data| {
            let json = serde_json::to_string(&data).unwrap();
            let _: TestData = serde_json::from_str(&json).unwrap();
        })
    }

    #[divan::bench]
    fn sonic_rs(bencher: divan::Bencher) {
        let data = TestData::sample();

        bencher.with_inputs(|| data.clone()).bench_values(|data| {
            let json = sonic_rs::to_string(&data).unwrap();
            let _: TestData = sonic_rs::from_str(&json).unwrap();
        })
    }
}

