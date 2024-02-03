#![cfg(feature = "ec_index")]

use std::{path::Path, thread};

use anyhow::Result;
use eccodes::{
    codes_index::Select, CodesError, CodesHandle, CodesIndex, FallibleStreamingIterator, KeyType,
    ProductKind,
};
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
fn collect_index_iterator() -> Result<()> {
    let keys = vec!["typeOfLevel", "level"];
    let index = CodesIndex::new_from_keys(&keys)?;
    let grib_path = Path::new("./data/iceland-levels.grib");

    let index = index
        .add_grib_file(grib_path)?
        .select("typeOfLevel", "isobaricInhPa")?
        .select("level", 700)?;

    let mut handle = CodesHandle::new_from_index(index, ProductKind::GRIB)?;

    let mut levels = vec![];

    while let Some(msg) = handle.next()? {
        levels.push(msg.clone());
    }

    assert_eq!(levels.len(), 5);

    Ok(())
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
fn simulatenous_index_destructors() -> Result<()> {
    let h1 = thread::spawn(|| -> anyhow::Result<(), CodesError> {
        let mut rng = rand::thread_rng();
        let file_path = Path::new("./data/iceland-surface.idx");

        for _ in 0..10 {
            let sleep_time = rng.gen_range(1..30); // randomizing sleep time to hopefully catch segfaults

            let index_op = CodesIndex::read_from_file(file_path)?
                .select("shortName", "2t")?
                .select("typeOfLevel", "surface")?
                .select("level", 0)?
                .select("stepType", "instant")?;

            thread::sleep(std::time::Duration::from_millis(sleep_time));
            drop(index_op);
        }

        Ok(())
    });

    let h2 = thread::spawn(|| -> anyhow::Result<(), CodesError> {
        let mut rng = rand::thread_rng();
        let keys = vec!["shortName", "typeOfLevel", "level", "stepType"];
        let grib_path = Path::new("./data/iceland-surface.grib");

        for _ in 0..10 {
            let sleep_time = rng.gen_range(1..42); // randomizing sleep time to hopefully catch segfaults

            let index = CodesIndex::new_from_keys(&keys)?
                .add_grib_file(grib_path)?
                .select("shortName", "2t")?
                .select("typeOfLevel", "surface")?
                .select("level", 0)?
                .select("stepType", "instant")?;

            thread::sleep(std::time::Duration::from_millis(sleep_time));
            drop(index);
        }

        Ok(())
    });

    h1.join().unwrap()?;
    h2.join().unwrap()?;

    Ok(())
}

#[test]
fn index_handle_interference() {
    thread::spawn(|| {
        let file_path = Path::new("./data/iceland.grib");

        loop {
            let handle = CodesHandle::new_from_file(file_path, ProductKind::GRIB);

            assert!(handle.is_ok());
        }
    });

    let mut rng = rand::thread_rng();
    let keys = vec!["shortName", "typeOfLevel", "level", "stepType"];
    let grib_path = Path::new("./data/iceland.grib");

    for _ in 0..10 {
        let sleep_time = rng.gen_range(1..42); // randomizing sleep time to hopefully catch segfaults

        let index = CodesIndex::new_from_keys(&keys)
            .unwrap()
            .add_grib_file(grib_path)
            .unwrap();
        let i_handle = CodesHandle::new_from_index(index, ProductKind::GRIB);

        assert!(i_handle.is_ok());

        thread::sleep(std::time::Duration::from_millis(sleep_time));
    }
}
