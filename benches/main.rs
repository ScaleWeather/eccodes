use std::path::Path;
use fallible_iterator::FallibleIterator;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use eccodes::codes_handle::{CodesHandle, ProductKind};

pub fn key_reading(c: &mut Criterion) {
    //prepare the variables for benchmark
    let file_path = Path::new("./data/iceland.grib");
    let product_kind = ProductKind::GRIB;

    let mut handle = CodesHandle::new_from_file(file_path, product_kind).unwrap();

    let msg = handle.next().unwrap().unwrap();

    c.bench_function("array reading", |b| {
        b.iter(|| msg.read_key(black_box("values")).unwrap())
    });

    c.bench_function("double reading", |b| {
        b.iter(|| {
            msg.read_key(black_box("jDirectionIncrementInDegrees"))
                .unwrap()
        })
    });
}

criterion_group!(benches, key_reading);
criterion_main!(benches);
