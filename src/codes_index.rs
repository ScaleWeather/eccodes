#![cfg_attr(docsrs, doc(cfg(feature = "experimental_index")))]
//! ⚠️ **EXPERIMENTAL FEATURE - POSSIBLY UNSAFE** ⚠️ \
//! Definition of `CodesIndex` and associated functions
//! used for efficient selection of messages from GRIB file
//!
//! # Safety Issues
//!
//! To understand the issue it's best to compare how `CodesIndex` and `CodesHandle` are created.
//!
//! Low-level [`*codes_handle`](eccodes_sys::codes_handle) in ecCodes is created from [`*FILE`](libc::FILE).
//! This crate utilises that by opening a file as [`fs::File`](std::fs::File) (which is memory and thread safe)
//! and passing to the ecCodes library [`RawFd`](std::os::fd::RawFd) which has the same safety guarantees
//! as `fs::File` as long as the file is in the scope. Therefore internal ecCodes file operations
//! in `CodesHandle` are safe-guarded by Rust filesystem API.
//!
//! In contrast, low-level ecCodes functions creating and manipulating [`*codes_index`](eccodes_sys::codes_index)
//! that perform IO operations take file paths as arguments instead of `*FILE` and no Rust IO safe guards
//! can be provided for `CodesIndex`. Therefore in concurrent contexts, when two ecCodes functions operate
//! on the same file (index or grib) on or both fail in an unpredictable way, leading to non-zero return codes
//! at best and segfaults at worst.
//!
//! This problem affects all functions in this module, and in other modules (eg. `CodesHandle::new_from_index`) if used
//! simulatenously with one of `codes_index` functions.
//!
//! The issues have been partially mitigated by implementing global mutex for `codes_index` operations.
//! Please not that mutex is used only for `codes_index` functions to not affect performance of other not-problematic functions in this crate.
//! This solution, eliminated tsegfaults in tests, but occasional non-zero return codes still appear. However, this is
//! not a guarantee and possbility of safety issues is non-zero!
//!
//! To avoid the memory issues altogether, do not use this feature at all. If you want to use it, take care to use `CodesIndex` in entirely
//! non-concurrent environment.
//!
//! If you have any suggestions or ideas how to improve the safety of this feature, please open an issue or a pull request.

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
/// Structure representing an index of messages inside a GRIB file
///
/// ⚠️ **WARNING** ⚠️
/// This feature might be unsafe to use, especially in concurrent environments.
/// You should not this feature in any production-like environment (eg. scheduled cluster jobs, web servers).
/// You can read about the issues in [module-level](crate::codes_index) documentation.
///
/// As GRIB files are binary dumps of message sections, index files encode the precalculated binary location
/// of messages matching a set of keys. Some GRIB files contain thousands of messages, and to facilitate
/// the fast reading of messages, index files are delivered with the files and used to query the binary
/// location of a message and receive handles. Loading GRIB files by reading an index file is required
/// for performance in certain situations.
///
/// Typical workflow for using `CodesIndex` involves:
/// - creating an index by reading file or constructing an empty one using [`new_from_keys`](CodesIndex::new_from_keys) or [`read_from_file`](CodesIndex::read_from_file)
/// - adding GRIB files to the index using [`add_grib_file`](CodesIndex::add_grib_file) (not required if the index is read from file)
/// - selecting messages by key-value pairs using [`select`](Select::select)
/// - reading the messages from the GRIB file by creating `CodesHandle` using [`CodesHandle::new_from_index`](crate::CodesHandle::new_from_index)
///
/// # Example
///
/// ```
/// # use std::path::Path;
/// # use eccodes::codes_index::{CodesIndex, Select};
/// # use eccodes::codes_handle::CodesHandle;
/// # fn main() -> anyhow::Result<()> {
/// let keys = vec!["shortName", "typeOfLevel"];
/// let grib_path = Path::new("./data/iceland.grib");
/// let index = CodesIndex::new_from_keys(&keys)?
///     .add_grib_file(grib_path)?
///     .select("shortName", "2t")?
///     .select("typeOfLevel", "surface")?;
///
/// let handle = CodesHandle::new_from_index(index)?;
/// # Ok(())
/// # }
/// ```
pub struct CodesIndex {
    pub(crate) pointer: *mut codes_index,
}

/// Selection of messages from the [`CodesIndex`] by key-value pairs. [`CodesHandle`](crate::codes_handle::CodesHandle)
/// created from the index will contain only messages matching the selection.
#[cfg_attr(docsrs, doc(cfg(feature = "experimental_index")))]
pub trait Select<T> {
    /// Selects messages from the index by key-value pair.
    /// Key must be a valid key present in the `CodesIndex` and value must be of the same type as the key.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::path::Path;
    /// # use eccodes::codes_index::{CodesIndex, Select};
    /// # fn main() -> anyhow::Result<()> {
    /// let idx_path = Path::new("./data/iceland-surface.grib.idx");
    /// let index = CodesIndex::read_from_file(idx_path)?
    ///     .select("shortName", "2t")?
    ///     .select("typeOfLevel", "surface")?
    ///     .select("level", 0)?
    ///     .select("stepType", "instant")?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return [`CodesError::Internal`] if the selection cannot be performed.
    fn select(self, key: &str, value: T) -> Result<CodesIndex, CodesError>;
}

impl CodesIndex {
    /// Constructs a new `CodesIndex` without an attached GRIB file.
    ///
    /// The function takes as an argument a list of keys that will be (exclusively) used to select messages.
    /// To use the index, you need to add GRIB files using [`add_grib_file`](CodesIndex::add_grib_file) method.
    ///
    /// # Example
    ///
    /// ```
    /// # use eccodes::codes_index::CodesIndex;
    /// # fn main() -> anyhow::Result<()> {
    /// let keys = ["shortName", "typeOfLevel", "level", "stepType"];
    /// let index = CodesIndex::new_from_keys(&keys)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return [`CodesError::Internal`] if the index cannot be created.
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

    /// Constructs a new `CodesIndex` by reading an index file at given path.
    ///
    /// The path must point to a valid index file created by ecCodes library.
    /// GRIB file corresponding to the index file must be present in the same relative path
    /// as during the index file creation.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::path::Path;
    /// # use eccodes::codes_index::CodesIndex;
    /// # fn main() -> anyhow::Result<()> {
    /// let file_path = Path::new("./data/iceland-surface.grib.idx");
    /// let index = CodesIndex::read_from_file(file_path)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return [`CodesError::Internal`] if the index file is not valid or
    /// the GRIB file is not present in the same relative path as during the index file creation.
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

    /// Attaches a GRIB file to the index.
    ///
    /// Path must point to a valid GRIB file.
    ///
    /// The function will index the file and add it to the index. There might be a performance
    /// overhead when adding a big file. Multiple files can be added to the index.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::path::Path;
    /// # use eccodes::codes_index::CodesIndex;
    /// # fn main() -> anyhow::Result<()> {
    /// let keys = vec!["shortName", "typeOfLevel", "level", "stepType"];
    /// let index = CodesIndex::new_from_keys(&keys)?;
    /// let grib_path = Path::new("./data/iceland.grib");
    /// let index = index.add_grib_file(grib_path)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`CodesError::Internal`] if the file cannot be added to the index.
    /// The error might be also caused by incorrectly constructed index.
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

#[doc(hidden)]
impl Drop for CodesIndex {
    fn drop(&mut self) {
        self.spec_drop();
    }
}

#[cfg(test)]
mod tests {
    use anyhow::{bail, Result};

    use crate::{
        codes_index::{CodesIndex, Select},
        errors::CodesInternal,
        CodesError,
    };
    use std::path::Path;
    #[test]
    fn index_constructors() -> Result<()> {
        {
            let keys = ["shortName", "typeOfLevel", "level", "stepType"];
            let index = CodesIndex::new_from_keys(&keys)?;
            assert!(!index.pointer.is_null());
        }
        {
            let file_path = Path::new("./data/iceland-surface.grib.idx");
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
        let file_path = Path::new("./data/iceland-surface.grib.idx");
        let index = CodesIndex::read_from_file(file_path)?
            .select("shortName", "2t")?
            .select("typeOfLevel", "surface")?
            .select("level", 0)?
            .select("stepType", "instant")?;

        assert!(!index.pointer.is_null());
        Ok(())
    }

    #[test]
    fn incorrect_index_path() -> Result<()> {
        let file_path = Path::new("./data/iceland-levels-bad-path.grib.idx");
        let index = CodesIndex::read_from_file(file_path);

        if let Err(CodesError::Internal(err)) = index {
            assert_eq!(err, CodesInternal::CodesIoProblem);
        } else {
            bail!("Expected CodesError::Internal(CodesInternal::CodesIoProblem)");
        }
        Ok(())
    }
}
