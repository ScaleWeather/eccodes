#![cfg_attr(docsrs, doc(cfg(feature = "experimental_index")))]
//! ⚠️ **EXPERIMENTAL FEATURE - POSSIBLY UNSAFE** ⚠️ \
//! Definition of `CodesIndex` and associated functions
//! used for efficient selection of messages from GRIB file

use crate::{
    codes_handle::SpecialDrop,
    errors::CodesError,
    intermediate_bindings::{
        codes_index_add_file, codes_index_new, codes_index_read, codes_index_select_double,
        codes_index_select_long, codes_index_select_string,
    },
};
use eccodes_sys::codes_index;
use std::path::Path;

#[derive(Debug)]
#[cfg_attr(docsrs, doc(cfg(feature = "experimental_index")))]
pub struct CodesIndex {
    pub(crate) pointer: *mut codes_index,
}

#[cfg_attr(docsrs, doc(cfg(feature = "experimental_index")))]
pub trait Select<T> {
    fn select(self, key: &str, value: T) -> Result<CodesIndex, CodesError>;
}

impl CodesIndex {
    #[cfg_attr(docsrs, doc(cfg(feature = "experimental_index")))]
    pub fn new_from_keys(keys: &[&str]) -> Result<CodesIndex, CodesError> {
        let keys = keys.join(",");

        let index_handle;
        unsafe {
            index_handle = codes_index_new(&keys)?;
        }
        Ok(CodesIndex {
            pointer: index_handle,
        })
    }

    #[cfg_attr(docsrs, doc(cfg(feature = "experimental_index")))]
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

    #[cfg_attr(docsrs, doc(cfg(feature = "experimental_index")))]
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
    use anyhow::Result;

    use crate::codes_index::{CodesIndex, Select};
    use std::path::Path;
    #[test]
    fn index_constructors() -> Result<()> {
        {
            let keys = vec!["shortName", "typeOfLevel", "level", "stepType"];
            let index = CodesIndex::new_from_keys(&keys)?;
            assert!(!index.pointer.is_null());
        }
        {
            let file_path = Path::new("./data/iceland-surface.idx");
            let index = CodesIndex::read_from_file(file_path)?;
            assert!(!index.pointer.is_null());
        }

        Ok(())
    }

    #[test]
    fn index_destructor() -> Result<()> {
        let keys = vec!["shortName", "typeOfLevel", "level", "stepType"];
        let index = CodesIndex::new_from_keys(&keys)?;

        drop(index);
        Ok(())
    }

    #[test]
    fn add_file() -> Result<()> {
        let keys = vec!["shortName", "typeOfLevel", "level", "stepType"];
        let index = CodesIndex::new_from_keys(&keys)?;
        let grib_path = Path::new("./data/iceland.grib");
        let index = index.add_grib_file(grib_path)?;

        assert!(!index.pointer.is_null());
        Ok(())
    }

    #[test]
    fn index_selection() -> Result<()> {
        let file_path = Path::new("./data/iceland-surface.idx");
        let index = CodesIndex::read_from_file(file_path)?
            .select("shortName", "2t")?
            .select("typeOfLevel", "surface")?
            .select("level", 0)?
            .select("stepType", "instant")?;

        assert!(!index.pointer.is_null());
        Ok(())
    }
}
