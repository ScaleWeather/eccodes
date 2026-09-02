#![allow(missing_docs)]
use criterion::{Criterion, criterion_group, criterion_main};
use eccodes::codes_file::{CodesFile, ProductKind};
use eccodes::{FallibleIterator, KeyRead};
use std::hint::black_box;
use std::path::Path;

pub fn key_reading(c: &mut Criterion) {
    //prepare the variables for benchmark
    let file_path = Path::new("./data/iceland.grib");
    let product_kind = ProductKind::GRIB;

    let mut handle = CodesFile::new_from_file(file_path, product_kind).unwrap();

    let msg = handle.ref_message_iter().next().unwrap().unwrap();

    c.bench_function("long reading", |b| {
        b.iter(|| -> i64 { msg.read_key(black_box("dataDate")).unwrap() })
    });

    c.bench_function("float reading", |b| {
        b.iter(|| -> f32 {
            msg.read_key(black_box("jDirectionIncrementInDegrees"))
                .unwrap()
        })
    });

    c.bench_function("double reading", |b| {
        b.iter(|| -> f64 {
            msg.read_key(black_box("jDirectionIncrementInDegrees"))
                .unwrap()
        })
    });

    c.bench_function("string reading", |b| {
        b.iter(|| -> String { msg.read_key(black_box("name")).unwrap() })
    });

    c.bench_function("long array reading", |b| {
        b.iter(|| -> Vec<i64> {
            msg.read_key(black_box("numberOfPointsAlongAParallel"))
                .unwrap()
        })
    });

    c.bench_function("float array reading", |b| {
        b.iter(|| -> Vec<f32> { msg.read_key(black_box("values")).unwrap() })
    });

    c.bench_function("double array reading", |b| {
        b.iter(|| -> Vec<f64> { msg.read_key(black_box("values")).unwrap() })
    });
}

criterion_group!(benches, key_reading);
criterion_main!(benches);
