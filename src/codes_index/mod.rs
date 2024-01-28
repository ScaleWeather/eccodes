//!Main crate module containing definition of `CodesIndex`
//!and all associated functions and data structures

use crate::{
    codes_handle::SpecialDrop,
    errors::CodesError,
    intermediate_bindings::codes_index::{
        codes_index_add_file, codes_index_new, codes_index_read, codes_index_select_double,
        codes_index_select_long, codes_index_select_string,
    },
};
use eccodes_sys::codes_index;
use std::path::Path;

#[derive(Debug)]
#[cfg_attr(docsrs, doc(cfg(feature = "ec_index")))]
pub struct CodesIndex {
    pub(crate) pointer: *mut codes_index,
}
pub trait Select<T> {
    fn select(self, key: &str, value: T) -> Result<CodesIndex, CodesError>;
}

impl CodesIndex {
    #[cfg_attr(docsrs, doc(cfg(feature = "ec_index")))]
    pub fn new_from_keys(keys: &[&str]) -> Result<CodesIndex, CodesError> {
        let keys = keys.join(",");

        let index_handle;
        unsafe {
            // technically codes_index_new can also select keys
            // but that would unnecessarily diverge the API
            // and would be error prone
            index_handle = codes_index_new(&keys)?;
        }
        Ok(CodesIndex {
            pointer: index_handle,
        })
    }

    #[cfg_attr(docsrs, doc(cfg(feature = "ec_index")))]
    pub fn read_from_file(index_file_path: &Path) -> Result<CodesIndex, CodesError> {
        let file_path = index_file_path.to_str().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Path is not valid utf8")
        })?;

        let index_handle;
        unsafe {
            index_handle = codes_index_read(file_path)?;
        }

        Ok(CodesIndex {
            pointer: index_handle,
        })
    }

    /// **WARNING: Trying to add GRIB file to a CodesIndex while GRIB's index file (or GRIB itself)
    /// is in use can cause segfault, panic or error.**
    #[cfg_attr(docsrs, doc(cfg(feature = "ec_index")))]
    pub fn add_grib_file(self, index_file_path: &Path) -> Result<CodesIndex, CodesError> {
        let file_path = index_file_path.to_str().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Path is not valid utf8")
        })?;

        let new_index = self;

        unsafe {
            codes_index_add_file(new_index.pointer, file_path)?;
        }

        Ok(new_index)
    }
}

impl Select<i64> for CodesIndex {
    fn select(self, key: &str, value: i64) -> Result<CodesIndex, CodesError> {
        let new_index = self;
        unsafe {
            codes_index_select_long(new_index.pointer, key, value)?;
        }

        Ok(new_index)
    }
}
impl Select<f64> for CodesIndex {
    fn select(self, key: &str, value: f64) -> Result<CodesIndex, CodesError> {
        let new_index = self;
        unsafe {
            codes_index_select_double(new_index.pointer, key, value)?;
        }
        Ok(new_index)
    }
}
impl Select<&str> for CodesIndex {
    fn select(self, key: &str, value: &str) -> Result<CodesIndex, CodesError> {
        let new_index = self;
        unsafe {
            codes_index_select_string(new_index.pointer, key, value)?;
        }
        Ok(new_index)
    }
}

impl Drop for CodesIndex {
    fn drop(&mut self) {
        self.spec_drop();
    }
}

#[cfg(test)]
mod tests {
    use fallible_iterator::FallibleIterator;

    use crate::{
        codes_index::{CodesIndex, Select},
        CodesHandle,
    };
    use crate::{KeyType, ProductKind};
    use std::path::Path;
    #[test]
    fn index_constructors() {
        {
            let keys = vec!["shortName", "typeOfLevel", "level", "stepType"];
            let index = CodesIndex::new_from_keys(&keys).unwrap();
            assert!(!index.pointer.is_null());
        }
        {
            let file_path = Path::new("./data/iceland-surface.idx");
            let index = CodesIndex::read_from_file(file_path).unwrap();
            assert!(!index.pointer.is_null());
        }
    }

    #[test]
    fn index_destructor() {
        let keys = vec!["shortName", "typeOfLevel", "level", "stepType"];
        let index = CodesIndex::new_from_keys(&keys).unwrap();

        drop(index)
    }

    #[test]
    fn add_file() {
        let keys = vec!["shortName", "typeOfLevel", "level", "stepType"];
        let index = CodesIndex::new_from_keys(&keys).unwrap();
        let grib_path = Path::new("./data/iceland.grib");
        let index = index.add_grib_file(grib_path).unwrap();

        assert!(!index.pointer.is_null());
    }

    #[test]
    fn index_selection() {
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

        assert!(!index.pointer.is_null());
    }

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
}
