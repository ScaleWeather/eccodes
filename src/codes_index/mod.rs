//!Main crate module containing definition of `CodesIndex`
//!and all associated functions and data structures

use crate::{
    errors::CodesError,
    intermediate_bindings::{
        codes_index_delete, codes_index_read, codes_index_select_double, codes_index_select_long,
        codes_index_select_string,
    },
};
use eccodes_sys::codes_index;
use log::warn;
use std::path::Path;

#[derive(Debug)]
pub struct CodesIndex {
    pub(crate) index_handle: *mut codes_index,
}
pub trait Select<T> {
    fn select(&mut self, key: &str, value: T) -> Result<(), CodesError>;
}

impl CodesIndex {
    #[cfg_attr(docsrs, doc(cfg(feature = "ec_index")))]
    pub fn new_from_file(file_path: &Path) -> Result<Self, CodesError> {
        let file_path_str = file_path.to_str().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Path is not valid utf8")
        })?;
        let index_handle;
        unsafe {
            index_handle = codes_index_read(file_path_str)?;
        }
        Ok(CodesIndex { index_handle })
    }
}

impl Select<i64> for CodesIndex {
    fn select(&mut self, key: &str, value: i64) -> Result<(), CodesError> {
        unsafe {
            codes_index_select_long(self.index_handle, key, value)?;
        }
        Ok(())
    }
}
impl Select<f64> for CodesIndex {
    fn select(&mut self, key: &str, value: f64) -> Result<(), CodesError> {
        unsafe {
            codes_index_select_double(self.index_handle, key, value)?;
        }
        Ok(())
    }
}
impl Select<&str> for CodesIndex {
    fn select(&mut self, key: &str, value: &str) -> Result<(), CodesError> {
        unsafe {
            codes_index_select_string(self.index_handle, key, value)?;
        }
        Ok(())
    }
}

impl Drop for CodesIndex {
    fn drop(&mut self) {
        unsafe {
            codes_index_delete(self.index_handle).unwrap_or_else(|error| {
                warn!("codes_index_delete() returned an error: {:?}", &error);
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::KeyType;
    use crate::{
        codes_index::{CodesIndex, Select},
        CodesHandle, KeyedMessage,
    };
    use std::{borrow::Borrow, path::Path};
    #[test]
    fn file_constructor() {
        let file_path = Path::new("./data/iceland-surface.idx");
        let index = CodesIndex::new_from_file(file_path).unwrap();
        assert!(!index.index_handle.is_null());
    }

    #[test]
    fn grib_handle_from_index_selection() {
        let file_path = Path::new("./data/iceland-surface.idx");
        let mut index = CodesIndex::new_from_file(file_path).unwrap();
        index.select("shortName", "2t").unwrap();
        index.select("typeOfLevel", "surface").unwrap();
        index.select("level", 0).unwrap();
        index.select("stepType", "instant").unwrap();
        let mut handle: CodesHandle = index.borrow().try_into().unwrap();
        let current_message: KeyedMessage = handle.try_into().unwrap();

        let short_name = current_message.read_key("shortName").unwrap();
        match short_name.value {
            KeyType::Str(val) => assert!(val == "2t"),
            _ => panic!("Unexpected key type"),
        };
        let level = current_message.read_key("level").unwrap();
        match level.value {
            KeyType::Int(val) => assert!(val == 0),
            _ => panic!("Unexpected key type"),
        };
        index.select("shortName", "10v").unwrap();
        handle = index.borrow().try_into().unwrap();
        let current_message: KeyedMessage = handle.try_into().unwrap();

        let short_name = current_message.read_key("shortName").unwrap();
        match short_name.value {
            KeyType::Str(val) => assert!(val == "10v"),
            _ => panic!("Unexpected key type"),
        };
    }
}
