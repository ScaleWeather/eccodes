use std::{path::Path, thread};

use anyhow::{Context, Result};
use eccodes::{CodesHandle, FallibleStreamingIterator, KeyType, ProductKind};

#[test]
fn thread_safety() -> Result<()> {
    thread::spawn(|| -> anyhow::Result<()> {
        loop {
            let file_path = Path::new("./data/iceland.grib");

            let mut handle = CodesHandle::new_from_file(file_path, ProductKind::GRIB)?;
            let current_message = handle.next()?.context("Message not some")?;

            for _ in 0..100 {
                let _ = current_message.read_key("name")?;

                let str_key = current_message.read_key("name")?;

                match str_key.value {
                    KeyType::Str(_) => {}
                    _ => panic!("Incorrect variant of string key"),
                }

                assert_eq!(str_key.name, "name");
            }

            drop(handle);
        }
    });

    for _ in 0..1000 {
        let file_path = Path::new("./data/iceland.grib");

        let mut handle = CodesHandle::new_from_file(file_path, ProductKind::GRIB)?;
        let current_message = handle.next()?.context("Message not some")?;

        let long_key = current_message.read_key("numberOfPointsAlongAParallel")?;

        match long_key.value {
            KeyType::Int(_) => {}
            _ => panic!("Incorrect variant of long key"),
        }

        assert_eq!(long_key.name, "numberOfPointsAlongAParallel");

        drop(handle);
    }

    Ok(())
}
