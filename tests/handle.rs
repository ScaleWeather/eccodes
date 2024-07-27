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

#[test]
fn check_no_testing_logs() -> Result<()> {
    testing_logger::setup();
    {
        let file_path = Path::new("./data/iceland-surface.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;

        let _ref_msg = handle.next()?.context("no message")?;
        let clone_msg = _ref_msg.try_clone()?;
        let _oth_ref = handle.next()?.context("no message")?;

        let _nrst = clone_msg.codes_nearest()?;
        let _kiter = clone_msg.default_keys_iterator()?;
    }

    testing_logger::validate(|captured_logs| {
        assert_eq!(captured_logs.len(), 0);
    });

    Ok(())
}
