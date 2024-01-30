#![cfg(feature = "ec_index")]

use std::{path::Path, thread};

use eccodes::{codes_index::Select, CodesHandle, CodesIndex, KeyType, ProductKind};
use fallible_iterator::FallibleIterator;
use rand::Rng;

#[test]
fn iterate_handle_from_index() {
    let file_path = Path::new("./data/iceland-surface.idx");
    let index = CodesIndex::read_from_file(file_path)
        .unwrap()
        .select("shortName", "2t")
        .unwrap()
        .select("typeOfLevel", "surface")
        .unwrap()
        .select("level", 0)
        .unwrap()
        .select("stepType", "instant")
        .unwrap();

    let handle = CodesHandle::new_from_index(index, ProductKind::GRIB).unwrap();

    let counter = handle.count().unwrap();

    assert_eq!(counter, 1);
}

#[test]
fn read_index_messages() {
    let file_path = Path::new("./data/iceland-surface.idx");
    let index = CodesIndex::read_from_file(file_path)
        .unwrap()
        .select("shortName", "2t")
        .unwrap()
        .select("typeOfLevel", "surface")
        .unwrap()
        .select("level", 0)
        .unwrap()
        .select("stepType", "instant")
        .unwrap();

    let mut handle = CodesHandle::new_from_index(index, ProductKind::GRIB).unwrap();
    let current_message = handle.next().unwrap().unwrap();

    {
        let short_name = current_message.read_key("shortName").unwrap();
        match short_name.value {
            KeyType::Str(val) => assert!(val == "2t"),
            _ => panic!("Unexpected key type"),
        };
    }
    {
        let level = current_message.read_key("level").unwrap();
        match level.value {
            KeyType::Int(val) => assert!(val == 0),
            _ => panic!("Unexpected key type"),
        };
    }
}

#[test]
fn collect_index_iterator() {
    let keys = vec!["typeOfLevel", "level"];
    let index = CodesIndex::new_from_keys(&keys).unwrap();
    let grib_path = Path::new("./data/iceland-levels.grib");

    let index = index
        .add_grib_file(grib_path)
        .unwrap()
        .select("typeOfLevel", "isobaricInhPa")
        .unwrap()
        .select("level", 700)
        .unwrap();

    let handle = CodesHandle::new_from_index(index, ProductKind::GRIB).unwrap();

    let level = handle.collect::<Vec<_>>().unwrap();

    assert_eq!(level.len(), 5);
}

#[test]
fn add_file_error() {
    thread::spawn(|| {
        let grib_path = Path::new("./data/iceland-levels.grib");
        let keys = vec!["shortName", "typeOfLevel", "level", "stepType"];
        let mut index_op = CodesIndex::new_from_keys(&keys).unwrap();

        loop {
            index_op = index_op.add_grib_file(grib_path).unwrap();
        }
    });

    thread::sleep(std::time::Duration::from_millis(250));

    let keys = vec!["shortName", "typeOfLevel", "level", "stepType"];
    let wrong_path = Path::new("./data/xxx.grib");
    let index = CodesIndex::new_from_keys(&keys)
        .unwrap()
        .add_grib_file(wrong_path);

    assert!(index.is_err());
}

#[test]
fn index_panic() {
    thread::spawn(|| {
        let grib_path = Path::new("./data/iceland-levels.grib");
        let keys = vec!["shortName", "typeOfLevel", "level", "stepType"];
        let mut index_op = CodesIndex::new_from_keys(&keys).unwrap();

        loop {
            index_op = index_op.add_grib_file(grib_path).unwrap();
        }
    });

    thread::sleep(std::time::Duration::from_millis(250));

    let keys = vec!["shortName", "typeOfLevel", "level", "stepType"];
    let wrong_path = Path::new("./data/xxx.grib");
    let index = CodesIndex::new_from_keys(&keys).unwrap();

    let result = std::panic::catch_unwind(|| index.add_grib_file(wrong_path).unwrap());

    assert!(result.is_err());
}

#[test]
fn add_file_while_index_open() {
    thread::spawn(|| {
        let file_path = Path::new("./data/iceland-surface.idx");
        let mut index_op = CodesIndex::read_from_file(file_path).unwrap();

        loop {
            index_op = index_op
                .select("shortName", "2t")
                .unwrap()
                .select("typeOfLevel", "surface")
                .unwrap()
                .select("level", 0)
                .unwrap()
                .select("stepType", "instant")
                .unwrap();
        }
    });

    let keys = vec!["shortName", "typeOfLevel", "level", "stepType"];
    let grib_path = Path::new("./data/iceland-surface.grib");
    let index = CodesIndex::new_from_keys(&keys)
        .unwrap()
        .add_grib_file(grib_path);

    assert!(index.is_ok());
}

#[test]
fn add_file_to_read_index() {
    let file_path = Path::new("./data/iceland-surface.idx");
    let grib_path = Path::new("./data/iceland-surface.grib");

    let _index = CodesIndex::read_from_file(file_path)
        .unwrap()
        .add_grib_file(grib_path)
        .unwrap()
        .select("shortName", "2t")
        .unwrap()
        .select("typeOfLevel", "surface")
        .unwrap()
        .select("level", 0)
        .unwrap()
        .select("stepType", "instant")
        .unwrap();
}

#[test]
fn simulatenous_index_destructors() {
    let h1 = thread::spawn(|| {
        let mut rng = rand::thread_rng();

        for _ in 0..10 {
            let sleep_time = rng.gen_range(12..50); // randomizing sleep time to hopefully catch segfaults

            let file_path = Path::new("./data/iceland-surface.idx");
            let index_op = CodesIndex::read_from_file(file_path)
                .unwrap()
                .select("shortName", "2t")
                .unwrap()
                .select("typeOfLevel", "surface")
                .unwrap()
                .select("level", 0)
                .unwrap()
                .select("stepType", "instant")
                .unwrap();

            thread::sleep(std::time::Duration::from_millis(sleep_time));
            drop(index_op);
        }
    });

    let h2 = thread::spawn(|| {
        let mut rng = rand::thread_rng();

        for _ in 0..10 {
            let sleep_time = rng.gen_range(24..65); // randomizing sleep time to hopefully catch segfaults

            let keys = vec!["shortName", "typeOfLevel", "level", "stepType"];
            let grib_path = Path::new("./data/iceland-surface.grib");
            let index = CodesIndex::new_from_keys(&keys)
                .unwrap()
                .add_grib_file(grib_path)
                .unwrap()
                .select("shortName", "2t")
                .unwrap()
                .select("typeOfLevel", "surface")
                .unwrap()
                .select("level", 0)
                .unwrap()
                .select("stepType", "instant")
                .unwrap();

            thread::sleep(std::time::Duration::from_millis(sleep_time));
            drop(index);
        }
    });

    h1.join().unwrap();
    h2.join().unwrap();
}
