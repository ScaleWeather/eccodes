use std::{path::Path, thread};

use eccodes::{CodesHandle, KeyType, ProductKind};
use fallible_iterator::FallibleIterator;

#[test]
fn thread_safety() {
    thread::spawn(|| loop {
        let file_path = Path::new("./data/iceland.grib");

        let mut handle = CodesHandle::new_from_file(file_path, ProductKind::GRIB).unwrap();
        let current_message = handle.next().unwrap().unwrap();

        for _ in 0..100 {
            let _ = current_message.read_key("name").unwrap();

            let str_key = current_message.read_key("name").unwrap();

            match str_key.value {
                KeyType::Str(_) => {}
                _ => panic!("Incorrect variant of string key"),
            }

            assert_eq!(str_key.name, "name");
        }

        drop(current_message);
        drop(handle);
    });

    for _ in 0..1000 {
        let file_path = Path::new("./data/iceland.grib");

        let mut handle = CodesHandle::new_from_file(file_path, ProductKind::GRIB).unwrap();
        let current_message = handle.next().unwrap().unwrap();

        let long_key = current_message
            .read_key("numberOfPointsAlongAParallel")
            .unwrap();

        match long_key.value {
            KeyType::Int(_) => {}
            _ => panic!("Incorrect variant of long key"),
        }

        assert_eq!(long_key.name, "numberOfPointsAlongAParallel");

        drop(current_message);
        drop(handle);
    }
}
