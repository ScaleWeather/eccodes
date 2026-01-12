//! Definition and constructors of `CodesFile`
//! used for accessing GRIB files

use crate::{CodesError, intermediate_bindings::codes_handle_new_from_file, pointer_guard};
use eccodes_sys::{ProductKind_PRODUCT_GRIB, codes_handle};
use errno::errno;
use libc::{FILE, c_char, c_void, size_t};
use std::{
    fmt::Debug,
    fs::{File, OpenOptions},
    os::unix::prelude::AsRawFd,
    path::Path,
};
use tracing::instrument;

pub use iterator::{ArcMessageIter, RefMessageIter};

mod iterator;

/// Structure providing access to the GRIB file which takes a full ownership of the accessed file.
///  
/// It can be constructed from:
///
/// - File path using [`new_from_file()`](CodesFile::new_from_file)
/// - From memory buffer using [`new_from_memory()`](CodesFile::new_from_memory)
///
/// Destructor for this structure does not panic, but some internal functions may rarely fail
/// leading to bugs. Errors encountered in the destructor are logged with [`tracing`].
/// 
/// To access GRIB messages the ecCodes library uses a method similar to a C-style iterator.
/// It digests the `* FILE` multiple times, each time returning the `*mut codes_handle`
/// to a message inside the file. Therefore in this crate access messages from `CodesFile`
///  use [`ref_message_iter()`](CodesFile::ref_message_iter) or [`arc_message_iter()`](CodesFile::arc_message_iter).
#[derive(Debug)]
pub struct CodesFile<D: Debug> {
    // fields are dropped from top
    pointer: *mut FILE,
    product_kind: ProductKind,
    _data: D,
}

// 2024-07-26
// Previously CodesFile had implemented Drop which called libc::fclose()
// but that closed the file descriptor and interfered with rust's fs::file destructor.
//
// To my best understanding the purpose of destructor is to clear memory and remove
// any pointers that would be dangling.
//
// The only pointer that is handed out of CodesFile is &CodesMessage, which is tied
// to CodesFile through lifetimes, so if we destruct CodesFile that pointer is first
// destructed as well. Source pointer is only used internally so we don't need to worry about it.
//
// Clearing the memory is handled on ecCodes side by CodesMessage/CodesIndex destructors
// and on rust side by destructors of data_container we own.

impl<D: Debug> CodesFile<D> {
    fn generate_codes_handle(&mut self) -> Result<*mut codes_handle, CodesError> {
        unsafe { codes_handle_new_from_file(self.pointer, self.product_kind) }
    }
}

/// Enum representing the kind of product (file type) inside handled file.
/// Used to indicate to ecCodes how it should decode/encode messages.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum ProductKind {
    #[allow(missing_docs)]
    GRIB = ProductKind_PRODUCT_GRIB as isize,
}

impl CodesFile<File> {
    /// Opens file at given [`Path`] as selected [`ProductKind`] and contructs `CodesFile`.
    /// 
    /// ## Example
    /// 
    /// ```
    /// # use eccodes::{ProductKind, CodesFile};
    /// # use std::path::Path;
    /// # fn main() -> anyhow::Result<()> {
    /// let handle = CodesFile::new_from_file("./data/iceland.grib", ProductKind::GRIB)?;
    ///  # Ok(())
    ///  # }
    /// ```
    /// 
    /// The function creates [`fs::File`](std::fs::File) from provided path and  utilises
    /// [`fdopen()`](https://man7.org/linux/man-pages/man3/fdopen.3.html)
    /// to associate [`io::RawFd`](`std::os::unix::io::RawFd`)
    /// with a stream represented by [`libc::FILE`](https://docs.rs/libc/0.2.101/libc/enum.FILE.html) pointer.
    /// 
    /// The constructor takes as argument a [`path`](Path) instead of [`File`]
    /// to ensure that `fdopen()` uses the same mode as [`File`].
    /// 
    ///  The file stream and [`File`] are safely closed when `CodesFile` is dropped.
    /// 
    /// ## Errors
    /// Returns [`CodesError::FileHandlingInterrupted`] with [`io::Error`](std::io::Error)
    /// when the file cannot be opened.
    /// 
    /// Returns [`CodesError::LibcNonZero`] with [`errno`](errno::Errno) information
    /// when the stream cannot be created from the file descriptor.
    /// 
    /// Returns [`CodesError::Internal`] with error code
    /// when internal [`codes_handle`] cannot be created.
    #[instrument(level = "trace")]
    pub fn new_from_file<P: AsRef<Path> + Debug>(
        file_path: P,
        product_kind: ProductKind,
    ) -> Result<Self, CodesError> {
        let file = OpenOptions::new().read(true).open(file_path)?;
        let file_pointer = open_with_fdopen(&file)?;

        Ok(Self {
            _data: file,
            pointer: file_pointer,
            product_kind,
        })
    }
}
impl CodesFile<Vec<u8>> {
    /// Opens data in provided buffer as selected [`ProductKind`] and contructs `CodesFile`.
    /// 
    /// ## Example
    /// 
    /// ```
    /// # async fn run() -> anyhow::Result<()> {
    /// # use eccodes::{ProductKind, CodesFile};
    /// #
    /// let product_kind = ProductKind::GRIB;
    /// let file_data =
    ///     reqwest::get("https://github.com/ScaleWeather/eccodes/blob/main/data/iceland.grib?raw=true")
    ///         .await?
    ///         .bytes()
    ///         .await?
    ///         .to_vec();
    /// 
    /// let handle = CodesFile::new_from_memory(file_data, product_kind)?;
    ///  # Ok(())
    /// # }
    /// ```
    /// 
    /// The function associates data in memory with a stream
    /// represented by [`libc::FILE`](https://docs.rs/libc/0.2.101/libc/enum.FILE.html) pointer
    /// using [`fmemopen()`](https://man7.org/linux/man-pages/man3/fmemopen.3.html).
    /// 
    /// The constructor takes full ownership of the data inside buffer,
    /// which is safely dropped during the [`CodesFile`] drop.
    /// 
    /// ## Errors
    /// Returns [`CodesError::LibcNonZero`] with [`errno`](errno::Errno) information
    /// when the file stream cannot be created.
    /// 
    /// Returns [`CodesError::Internal`] with error code
    /// when internal [`codes_handle`] cannot be created.
    #[instrument(level = "trace")]
    pub fn new_from_memory(
        file_data: Vec<u8>,
        product_kind: ProductKind,
    ) -> Result<Self, CodesError> {
        let file_pointer = open_with_fmemopen(&file_data)?;

        Ok(Self {
            _data: file_data,
            product_kind,
            pointer: file_pointer,
        })
    }
}

#[instrument(level = "trace")]
fn open_with_fdopen(file: &File) -> Result<*mut FILE, CodesError> {
    let file_ptr = unsafe { libc::fdopen(file.as_raw_fd(), "r".as_ptr().cast::<_>()) };

    if file_ptr.is_null() {
        let error_val = errno();
        let error_code = error_val.0;
        return Err(CodesError::LibcNonZero(error_code, error_val));
    }

    Ok(file_ptr)
}

#[instrument(level = "trace")]
fn open_with_fmemopen(file_data: &[u8]) -> Result<*mut FILE, CodesError> {
    let file_data_ptr = file_data.as_ptr() as *mut c_void;
    pointer_guard::non_null!(file_data_ptr);

    let file_ptr;
    unsafe {
        file_ptr = libc::fmemopen(
            file_data_ptr,
            file_data.len() as size_t,
            "r".as_ptr().cast::<_>(),
        );
    }

    if file_ptr.is_null() {
        let error_val = errno();
        let error_code = error_val.0;
        return Err(CodesError::LibcNonZero(error_code, error_val));
    }

    Ok(file_ptr)
}

#[cfg(test)]
mod tests {
    use crate::codes_file::{CodesFile, ProductKind};
    use anyhow::{Context, Result};
    use eccodes_sys::ProductKind_PRODUCT_GRIB;
    use fallible_iterator::FallibleIterator;
    use std::{fs::File, io::Read, path::Path};

    #[test]
    fn file_constructor() -> Result<()> {
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let codes_file = CodesFile::new_from_file(file_path, product_kind)?;

        assert!(!codes_file.pointer.is_null());
        assert_eq!(codes_file.product_kind as u32, { ProductKind_PRODUCT_GRIB });

        codes_file._data.metadata()?;

        Ok(())
    }

    #[test]
    fn memory_constructor() -> Result<()> {
        let product_kind = ProductKind::GRIB;

        let mut f = File::open(Path::new("./data/iceland.grib"))?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;

        let codes_file = CodesFile::new_from_memory(buf, product_kind)?;
        assert!(!codes_file.pointer.is_null());
        assert_eq!(codes_file.product_kind as u32, { ProductKind_PRODUCT_GRIB });

        assert!(!codes_file._data.is_empty());

        Ok(())
    }

    #[test]
    fn codes_handle_drop_file() -> Result<()> {
        let file_path = Path::new("./data/iceland-surface.grib");
        let product_kind = ProductKind::GRIB;

        let handle = CodesFile::new_from_file(file_path, product_kind)?;
        drop(handle);

        Ok(())
    }

    #[test]
    fn codes_handle_drop_mem() -> Result<()> {
        let product_kind = ProductKind::GRIB;

        let mut f = File::open(Path::new("./data/iceland.grib"))?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;

        let handle = CodesFile::new_from_memory(buf, product_kind)?;
        drop(handle);

        Ok(())
    }

    #[test]
    fn multiple_drops() -> Result<()> {
        {
            let file_path = Path::new("./data/iceland-surface.grib");
            let product_kind = ProductKind::GRIB;

            let mut handle = CodesFile::new_from_file(file_path, product_kind)?;

            let _ref_msg = handle.ref_message_iter().next()?.context("no message")?;
            let mut clone_msg = _ref_msg.try_clone()?;
            drop(_ref_msg);
            let _oth_ref = handle.ref_message_iter().next()?.context("no message")?;

            let _nrst = clone_msg.codes_nearest()?;
            drop(_nrst);
            let _kiter = clone_msg.default_keys_iterator()?;
        }

        Ok(())
    }
}
