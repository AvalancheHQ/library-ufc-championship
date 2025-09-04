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
            values: vec![1, 2, 3, 4, 5, 10, 20, 30],
            metadata: Some("sample metadata".to_string()),
        }
    }
}

const DIVAN_MIN_TIME: Duration = Duration::from_secs(1);

fn main() {
    divan::main();
}

trait JsonSerializer: Sync {
    fn serialize(&self, data: &TestData) -> String;
    fn deserialize(&self, json: &str) -> TestData;
}

struct SerdeJsonImpl;
struct SonicRsImpl;

unsafe impl Sync for SerdeJsonImpl {}
unsafe impl Sync for SonicRsImpl {}

impl JsonSerializer for SerdeJsonImpl {
    fn serialize(&self, data: &TestData) -> String {
        serde_json::to_string(data).unwrap()
    }

    fn deserialize(&self, json: &str) -> TestData {
        serde_json::from_str(json).unwrap()
    }
}

impl JsonSerializer for SonicRsImpl {
    fn serialize(&self, data: &TestData) -> String {
        sonic_rs::to_string(data).unwrap()
    }

    fn deserialize(&self, json: &str) -> TestData {
        sonic_rs::from_str(json).unwrap()
    }
}

fn bench_serialize<T: JsonSerializer>(bencher: divan::Bencher, serializer: &T) {
    let data = TestData::sample();

    bencher
        .with_inputs(|| data.clone())
        .bench_values(|data| serializer.serialize(&data))
}

fn bench_deserialize<T: JsonSerializer>(bencher: divan::Bencher, serializer: &T) {
    let data = TestData::sample();
    let json = serializer.serialize(&data);

    bencher
        .with_inputs(|| json.clone())
        .bench_values(|json| serializer.deserialize(&json))
}

fn bench_serialize_deserialize_roundtrip<T: JsonSerializer>(
    bencher: divan::Bencher,
    serializer: &T,
) {
    let data = TestData::sample();

    bencher.with_inputs(|| data.clone()).bench_values(|data| {
        let json = serializer.serialize(&data);
        serializer.deserialize(&json)
    })
}

#[divan::bench(min_time = DIVAN_MIN_TIME)]
fn serialize_serde_json(bencher: divan::Bencher) {
    bench_serialize(bencher, &SerdeJsonImpl)
}

#[divan::bench(min_time = DIVAN_MIN_TIME)]
fn serialize_sonic_rs(bencher: divan::Bencher) {
    bench_serialize(bencher, &SonicRsImpl)
}

#[divan::bench(min_time = DIVAN_MIN_TIME)]
fn deserialize_serde_json(bencher: divan::Bencher) {
    bench_deserialize(bencher, &SerdeJsonImpl)
}

#[divan::bench(min_time = DIVAN_MIN_TIME)]
fn deserialize_sonic_rs(bencher: divan::Bencher) {
    bench_deserialize(bencher, &SonicRsImpl)
}

#[divan::bench(min_time = DIVAN_MIN_TIME)]
fn serialize_deserialize_roundtrip_serde_json(bencher: divan::Bencher) {
    bench_serialize_deserialize_roundtrip(bencher, &SerdeJsonImpl)
}

#[divan::bench(min_time = DIVAN_MIN_TIME)]
fn serialize_deserialize_roundtrip_sonic_rs(bencher: divan::Bencher) {
    bench_serialize_deserialize_roundtrip(bencher, &SonicRsImpl)
}
