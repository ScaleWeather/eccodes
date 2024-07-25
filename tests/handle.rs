use std::{path::Path, thread};

use anyhow::{Context, Result};
use eccodes::{CodesHandle, DynamicKeyType, FallibleStreamingIterator, ProductKind};

#[test]
fn thread_safety() {
    // errors are fine
    thread_safety_core().unwrap_or(());
}

fn thread_safety_core() -> Result<()> {
    thread::spawn(|| -> anyhow::Result<()> {
        loop {
            let file_path = Path::new("./data/iceland.grib");

            let mut handle = CodesHandle::new_from_file(file_path, ProductKind::GRIB)?;
            let current_message = handle.next()?.context("Message not some")?;

            for _ in 0..100 {
                let _ = current_message.read_key_dynamic("name")?;

                let str_key = current_message.read_key_dynamic("name")?;

                match str_key {
                    DynamicKeyType::Str(_) => {}
                    _ => panic!("Incorrect variant of string key"),
                }
            }

            drop(handle);
        }
    });

    for _ in 0..1000 {
        let file_path = Path::new("./data/iceland.grib");

        let mut handle = CodesHandle::new_from_file(file_path, ProductKind::GRIB)?;
        let current_message = handle.next()?.context("Message not some")?;

        let long_key = current_message.read_key_dynamic("numberOfPointsAlongAParallel")?;

        match long_key {
            DynamicKeyType::Int(_) => {}
            _ => panic!("Incorrect variant of long key"),
        }

        drop(handle);
    }

    Ok(())
}
