#![cfg(feature = "experimental_index")]

use std::{
    path::Path,
    sync::{Arc, Barrier},
    thread,
};

use anyhow::{Context, Result};
use eccodes::{
    codes_index::Select, CodesError, CodesHandle, CodesIndex, FallibleStreamingIterator, KeyOps,
    ProductKind,
};
use rand::Rng;

#[test]
fn iterate_handle_from_index() -> Result<()> {
    let file_path = Path::new("./data/iceland-surface.grib.idx");
    let index = CodesIndex::read_from_file(file_path)?
        .select("shortName", "2t")?
        .select("typeOfLevel", "surface")?
        .select("level", 0)?
        .select("stepType", "instant")?;

    let handle = CodesHandle::new_from_index(index)?;

    let counter = handle.count()?;

    assert_eq!(counter, 1);

    Ok(())
}

#[test]
fn read_index_messages() -> Result<()> {
    let file_path = Path::new("./data/iceland-surface.grib.idx");
    let index = CodesIndex::read_from_file(file_path)?
        .select("shortName", "2t")?
        .select("typeOfLevel", "surface")?
        .select("level", 0)?
        .select("stepType", "instant")?;

    let mut handle = CodesHandle::new_from_index(index)?;
    let current_message = handle.next()?.context("Message not some")?;

    {
        let short_name: String = current_message.read_key("shortName")?;
        assert_eq!(short_name, "2t");
    }
    {
        let level: i64 = current_message.read_key("level")?;
        assert_eq!(level, 0);
    }

    Ok(())
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

    let mut handle = CodesHandle::new_from_index(index)?;

    let mut levels = vec![];

    while let Some(msg) = handle.next()? {
        levels.push(msg.try_clone()?);
    }

    assert_eq!(levels.len(), 5);

    Ok(())
}

#[test]
fn add_file_error() -> Result<()> {
    thread::spawn(|| -> Result<()> {
        let grib_path = Path::new("./data/iceland-levels.grib");
        let keys = vec!["shortName", "typeOfLevel", "level", "stepType"];
        let mut index_op = CodesIndex::new_from_keys(&keys)?;

        loop {
            index_op = index_op.add_grib_file(grib_path)?;
        }
    });

    thread::sleep(std::time::Duration::from_millis(250));

    let keys = vec!["shortName", "typeOfLevel", "level", "stepType"];
    let wrong_path = Path::new("./data/xxx.grib");
    let index = CodesIndex::new_from_keys(&keys)?.add_grib_file(wrong_path);

    assert!(index.is_err());

    Ok(())
}

#[test]
fn index_panic() -> Result<()> {
    thread::spawn(|| -> Result<()> {
        let grib_path = Path::new("./data/iceland-levels.grib");
        let keys = vec!["shortName", "typeOfLevel", "level", "stepType"];
        let mut index_op = CodesIndex::new_from_keys(&keys)?;

        loop {
            index_op = index_op.add_grib_file(grib_path)?;
        }
    });

    thread::sleep(std::time::Duration::from_millis(250));

    let keys = vec!["shortName", "typeOfLevel", "level", "stepType"];
    let wrong_path = Path::new("./data/xxx.grib");
    let index = CodesIndex::new_from_keys(&keys)?;

    let result = std::panic::catch_unwind(|| index.add_grib_file(wrong_path).unwrap());

    assert!(result.is_err());

    Ok(())
}

#[test]
#[ignore = "for releases, indexing is experimental"]
fn add_file_while_index_open() -> Result<()> {
    thread::spawn(|| -> Result<()> {
        let file_path = Path::new("./data/iceland-surface.grib.idx");
        let mut index_op = CodesIndex::read_from_file(file_path)?;

        loop {
            index_op = index_op
                .select("shortName", "2t")?
                .select("typeOfLevel", "surface")?
                .select("level", 0)?
                .select("stepType", "instant")?;
        }
    });

    let keys = vec!["shortName", "typeOfLevel", "level", "stepType"];
    let grib_path = Path::new("./data/iceland-surface.grib");
    let index = CodesIndex::new_from_keys(&keys)?.add_grib_file(grib_path);

    assert!(index.is_ok());

    Ok(())
}

#[test]
fn add_file_to_read_index() -> Result<()> {
    let file_path = Path::new("./data/iceland-surface.grib.idx");
    let grib_path = Path::new("./data/iceland-surface.grib");

    let _index = CodesIndex::read_from_file(file_path)?
        .add_grib_file(grib_path)?
        .select("shortName", "2t")?
        .select("typeOfLevel", "surface")?
        .select("level", 0)?
        .select("stepType", "instant")?;

    Ok(())
}

#[test]
#[ignore = "for releases, indexing is experimental"]
fn simulatenous_index_destructors() -> Result<()> {
    let barrier = Arc::new(Barrier::new(2));
    let b1 = barrier.clone();
    let b2 = barrier.clone();

    let h1 = thread::spawn(move || -> anyhow::Result<(), CodesError> {
        let file_path = Path::new("./data/iceland-surface.grib.idx");

        for _ in 0..100 {
            let index_op = CodesIndex::read_from_file(file_path)?
                .select("shortName", "2t")?
                .select("typeOfLevel", "surface")?
                .select("level", 0)?
                .select("stepType", "instant")?;

            b1.wait();
            drop(index_op);
        }

        Ok(())
    });

    let h2 = thread::spawn(move || -> anyhow::Result<(), CodesError> {
        let keys = vec!["shortName", "typeOfLevel", "level", "stepType"];
        let grib_path = Path::new("./data/iceland-surface.grib");

        for _ in 0..100 {
            let index = CodesIndex::new_from_keys(&keys)?
                .add_grib_file(grib_path)?
                .select("shortName", "2t")?
                .select("typeOfLevel", "surface")?
                .select("level", 0)?
                .select("stepType", "instant")?;

            b2.wait();
            drop(index);
        }

        Ok(())
    });

    // errors are fine
    h1.join().unwrap().unwrap_or(());
    h2.join().unwrap().unwrap_or(());

    Ok(())
}

#[test]
#[ignore = "for releases, indexing is experimental"]
fn index_handle_interference() -> Result<()> {
    thread::spawn(|| -> Result<()> {
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

        let index = CodesIndex::new_from_keys(&keys)?.add_grib_file(grib_path)?;
        let i_handle = CodesHandle::new_from_index(index);

        assert!(i_handle.is_ok());

        thread::sleep(std::time::Duration::from_millis(sleep_time));
    }

    Ok(())
}
