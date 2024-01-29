#![cfg(feature = "ec_index")]

use std::path::Path;

use eccodes::{codes_index::Select, CodesHandle, CodesIndex, KeyType, ProductKind};
use fallible_iterator::FallibleIterator;

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