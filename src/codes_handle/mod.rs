//! Definition and constructors of `CodesHandle`
//! used for accessing GRIB files

#[cfg(feature = "experimental_index")]
use crate::codes_index::CodesIndex;
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

pub use iterator::{ArcMessageGenerator, RefMessageGenerator};

mod iterator;

/// This is an internal structure used to access provided file by `CodesHandle`.
/// It also allows to differentiate between `CodesHandle` created from file and from index.
/// It is not intended to be used directly by the user.
#[doc(hidden)]
#[derive(Debug)]
pub struct CodesFile<D: Debug> {
    // fields dropped from top
    pointer: *mut FILE,
    product_kind: ProductKind,
    _data: D,
}

/// Marker trait to differentiate between `CodesHandle` created from index and file/buffer.
#[doc(hidden)]
pub trait ThreadSafeHandle: HandleGenerator {}

impl ThreadSafeHandle for CodesFile<Vec<u8>> {}
impl ThreadSafeHandle for CodesFile<File> {}

/// Internal trait implemented for types that can be called to generate `*mut codes_handle`.
#[doc(hidden)]
pub trait HandleGenerator: Debug {
    fn gen_codes_handle(&self) -> Result<*mut codes_handle, CodesError>;
}

impl<D: Debug> HandleGenerator for CodesFile<D> {
    fn gen_codes_handle(&self) -> Result<*mut codes_handle, CodesError> {
        unsafe { codes_handle_new_from_file(self.pointer, self.product_kind) }
    }
}

/// Structure providing access to the GRIB file which takes a full ownership of the accessed file.
///  
/// It can be constructed from:
///
/// - File path using [`new_from_file()`](CodesHandle::new_from_file)
/// - From memory buffer using [`new_from_memory()`](CodesHandle::new_from_memory)
/// - From GRIB index using [`new_from_index()`](CodesHandle::new_from_index) (with `experimental_index` feature enabled)
///
/// Destructor for this structure does not panic, but some internal functions may rarely fail
/// leading to bugs. Errors encountered in the destructor are logged with [`tracing`].
///
/// # `FallibleIterator`
///
/// This structure implements [`FallibleIterator`](crate::FallibleStreamingIterator) trait which allows to access GRIB messages.
///
/// To access GRIB messages the ecCodes library uses a method similar to a C-style iterator.
/// It digests the `* FILE` multiple times, each time returning the `*mut codes_handle`
/// to a message inside the file.
///
/// This behaviour is represented in this crate by `FallibleIterator`, because generating `KeyedMessage` can fail.
///
/// ```
/// use eccodes::{ProductKind, CodesHandle, KeyRead};
/// # use std::path::Path;
/// // FallibleStreamingIterator must be in scope to use it
/// use eccodes::FallibleStreamingIterator;
/// #
/// # fn main() -> anyhow::Result<(), eccodes::errors::CodesError> {
/// let file_path = Path::new("./data/iceland-surface.grib");
/// let product_kind = ProductKind::GRIB;
///
/// let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;
///
/// // Print names of messages in the file
/// while let Some(message) = handle.next()? {
///     // The message must be unwraped as internal next() can fail
///     let key: String = message.read_key("name")?;
///     println!("{key}");    
///
/// }
/// # Ok(())
/// # }
/// ```
///
/// You can also manually collect the messages into a vector to use them later.
///
/// ```
/// use eccodes::{ProductKind, CodesHandle, KeyedMessage};
/// # use eccodes::errors::CodesError;
/// # use std::path::Path;
/// use eccodes::FallibleStreamingIterator;
/// #
/// # fn main() -> anyhow::Result<(), eccodes::errors::CodesError> {
/// let file_path = Path::new("./data/iceland-surface.grib");
/// let product_kind = ProductKind::GRIB;
///
/// let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;
///
/// let mut handle_collected = vec![];
///
/// while let Some(msg) = handle.next()? {
///     handle_collected.push(msg.try_clone()?);
/// }
/// # Ok(())
/// # }
/// ```
///
/// All available methods for `CodesHandle` iterator can be found in [`FallibleStreamingIterator`](crate::FallibleStreamingIterator) trait.
#[derive(Debug)]
pub struct CodesHandle<S: HandleGenerator> {
    source: S,
}

// 2024-07-26
// Previously CodesHandle had implemented Drop which called libc::fclose()
// but that closed the file descriptor and interfered with rust's fs::file destructor.
//
// To my best understanding the purpose of destructor is to clear memory and remove
// any pointers that would be dangling.
//
// The only pointer that is handed out of CodesHandle is &KeyedMessage, which is tied
// to CodesHandle through lifetimes, so if we destruct CodesHandle that pointer is first
// destructed as well. Source pointer is only used internally so we don't need to worry about it.
//
// Clearing the memory is handled on ecCodes side by KeyedMessage/CodesIndex destructors
// and on rust side by destructors of data_container we own.

/// Enum representing the kind of product (file type) inside handled file.
/// Used to indicate to ecCodes how it should decode/encode messages.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum ProductKind {
    #[allow(missing_docs)]
    GRIB = ProductKind_PRODUCT_GRIB as isize,
}

impl CodesHandle<CodesFile<File>> {
    ///Opens file at given [`Path`] as selected [`ProductKind`] and contructs `CodesHandle`.
    ///
    ///## Example
    ///
    ///```
    ///# use eccodes::codes_handle::{ProductKind, CodesHandle};
    ///# use std::path::Path;
    ///# fn main() -> anyhow::Result<()> {
    ///let file_path = Path::new("./data/iceland.grib");
    ///let product_kind = ProductKind::GRIB;
    ///
    ///let handle = CodesHandle::new_from_file(file_path, product_kind)?;
    /// # Ok(())
    /// # }
    ///```
    ///
    ///The function creates [`fs::File`](std::fs::File) from provided path and  utilises
    ///[`fdopen()`](https://man7.org/linux/man-pages/man3/fdopen.3.html)
    ///to associate [`io::RawFd`](`std::os::unix::io::RawFd`)
    ///with a stream represented by [`libc::FILE`](https://docs.rs/libc/0.2.101/libc/enum.FILE.html) pointer.
    ///
    ///The constructor takes as argument a [`path`](Path) instead of [`File`]
    ///to ensure that `fdopen()` uses the same mode as [`File`].
    ///
    /// The file stream and [`File`] are safely closed when `CodesHandle` is dropped.
    ///
    ///## Errors
    ///Returns [`CodesError::FileHandlingInterrupted`] with [`io::Error`](std::io::Error)
    ///when the file cannot be opened.
    ///
    ///Returns [`CodesError::LibcNonZero`] with [`errno`](errno::Errno) information
    ///when the stream cannot be created from the file descriptor.
    ///
    ///Returns [`CodesError::Internal`] with error code
    ///when internal [`codes_handle`] cannot be created.
    #[instrument(level = "trace")]
    pub fn new_from_file<P: AsRef<Path> + Debug>(
        file_path: P,
        product_kind: ProductKind,
    ) -> Result<Self, CodesError> {
        let file = OpenOptions::new().read(true).open(file_path)?;
        let file_pointer = open_with_fdopen(&file)?;

        Ok(Self {
            source: CodesFile {
                _data: file,
                pointer: file_pointer,
                product_kind,
            },
        })
    }
}
impl CodesHandle<CodesFile<Vec<u8>>> {
    ///Opens data in provided buffer as selected [`ProductKind`] and contructs `CodesHandle`.
    ///
    ///## Example
    ///
    ///```
    ///# async fn run() -> anyhow::Result<()> {
    ///# use eccodes::{ProductKind, CodesHandle};
    ///#
    ///let product_kind = ProductKind::GRIB;
    ///let file_data =
    ///    reqwest::get("https://github.com/ScaleWeather/eccodes/blob/main/data/iceland.grib?raw=true")
    ///        .await?
    ///        .bytes()
    ///        .await?
    ///        .to_vec();
    ///
    ///let handle = CodesHandle::new_from_memory(file_data, product_kind)?;
    /// # Ok(())
    ///# }
    ///```
    ///
    ///The function associates data in memory with a stream
    ///represented by [`libc::FILE`](https://docs.rs/libc/0.2.101/libc/enum.FILE.html) pointer
    ///using [`fmemopen()`](https://man7.org/linux/man-pages/man3/fmemopen.3.html).
    ///
    ///The constructor takes full ownership of the data inside buffer,
    ///which is safely dropped during the [`CodesHandle`] drop.
    ///
    ///## Errors
    ///Returns [`CodesError::LibcNonZero`] with [`errno`](errno::Errno) information
    ///when the file stream cannot be created.
    ///
    ///Returns [`CodesError::Internal`] with error code
    ///when internal [`codes_handle`] cannot be created.
    #[instrument(level = "trace")]
    pub fn new_from_memory(
        file_data: Vec<u8>,
        product_kind: ProductKind,
    ) -> Result<Self, CodesError> {
        let file_pointer = open_with_fmemopen(&file_data)?;

        Ok(Self {
            source: CodesFile {
                _data: file_data,
                product_kind,
                pointer: file_pointer,
            },
        })
    }
}

#[cfg(feature = "experimental_index")]
#[cfg_attr(docsrs, doc(cfg(feature = "experimental_index")))]
impl CodesHandle<CodesIndex> {
    /// Creates [`CodesHandle`] for provided [`CodesIndex`].
    ///
    /// ## Example
    ///
    /// ```
    /// # fn run() -> anyhow::Result<()> {
    /// # use eccodes::{CodesHandle, CodesIndex};
    /// #
    /// let index = CodesIndex::new_from_keys(&vec!["shortName", "typeOfLevel", "level"])?;
    /// let handle = CodesHandle::new_from_index(index)?;
    ///
    /// Ok(())
    /// # }
    /// ```
    ///
    /// The function takes ownership of the provided [`CodesIndex`] which owns
    /// the GRIB data. [`CodesHandle`] created from [`CodesIndex`] is of different type
    /// than the one created from file or memory buffer, because it internally uses
    /// different functions to access messages. But it can be used in the same way.
    ///
    /// ⚠️ Warning: This function may interfere with other functions in concurrent context,
    /// due to ecCodes issues with thread-safety for indexes. More information can be found
    /// in [`codes_index`](crate::codes_index) module documentation.
    ///
    /// ## Errors
    ///
    /// Returns [`CodesError::Internal`] with error code
    /// when internal [`codes_handle`] cannot be created.
    #[instrument(level = "trace")]
    pub fn new_from_index(index: CodesIndex) -> Result<Self, CodesError> {
        let new_handle = CodesHandle {
            source: index,
        };

        Ok(new_handle)
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
    use crate::codes_handle::{CodesHandle, ProductKind};
    #[cfg(feature = "experimental_index")]
    use crate::codes_index::{CodesIndex, Select};
    use anyhow::{Context, Result};
    use eccodes_sys::ProductKind_PRODUCT_GRIB;
    use fallible_iterator::FallibleIterator;
    use std::{fs::File, io::Read, path::Path};

    #[test]
    fn file_constructor() -> Result<()> {
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let handle = CodesHandle::new_from_file(file_path, product_kind)?;

        assert!(!handle.source.pointer.is_null());
        assert_eq!(handle.source.product_kind as u32, {
            ProductKind_PRODUCT_GRIB
        });

        handle.source._data.metadata()?;

        Ok(())
    }

    #[test]
    fn memory_constructor() -> Result<()> {
        let product_kind = ProductKind::GRIB;

        let mut f = File::open(Path::new("./data/iceland.grib"))?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;

        let handle = CodesHandle::new_from_memory(buf, product_kind)?;
        assert!(!handle.source.pointer.is_null());
        assert_eq!(handle.source.product_kind as u32, {
            ProductKind_PRODUCT_GRIB
        });

        assert!(!handle.source._data.is_empty());

        Ok(())
    }

    #[test]
    #[cfg(feature = "experimental_index")]
    fn index_constructor_and_destructor() -> Result<()> {
        use anyhow::Ok;

        let file_path = Path::new("./data/iceland-surface.grib.idx");
        let index = CodesIndex::read_from_file(file_path)?
            .select("shortName", "2t")?
            .select("typeOfLevel", "surface")?
            .select("level", 0)?
            .select("stepType", "instant")?;

        let i_ptr = index.pointer.clone();

        let handle = CodesHandle::new_from_index(index)?;

        assert_eq!(handle.source.pointer, i_ptr);
        assert!(handle.current_message.is_none());

        Ok(())
    }

    #[test]
    fn codes_handle_drop_file() -> Result<()> {
        let file_path = Path::new("./data/iceland-surface.grib");
        let product_kind = ProductKind::GRIB;

        let handle = CodesHandle::new_from_file(file_path, product_kind)?;
        drop(handle);

        Ok(())
    }

    #[test]
    fn codes_handle_drop_mem() -> Result<()> {
        let product_kind = ProductKind::GRIB;

        let mut f = File::open(Path::new("./data/iceland.grib"))?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;

        let handle = CodesHandle::new_from_memory(buf, product_kind)?;
        drop(handle);

        Ok(())
    }

    #[test]
    fn multiple_drops() -> Result<()> {
        {
            let file_path = Path::new("./data/iceland-surface.grib");
            let product_kind = ProductKind::GRIB;

            let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;

            let _ref_msg = handle
                .ref_message_generator()
                .next()?
                .context("no message")?;
            let clone_msg = _ref_msg.try_clone()?;
            drop(_ref_msg);
            let _oth_ref = handle
                .ref_message_generator()
                .next()?
                .context("no message")?;

            let _nrst = clone_msg.codes_nearest()?;
            let _kiter = clone_msg.default_keys_iterator()?;
        }

        Ok(())
    }

    #[test]
    #[cfg(feature = "experimental_index")]
    fn codes_handle_drop_index() -> Result<()> {
        testing_logger::setup();

        let file_path = Path::new("./data/iceland-surface.grib.idx");
        let index = CodesIndex::read_from_file(file_path)?;
        assert!(!index.pointer.is_null());

        let handle = CodesHandle::new_from_index(index)?;
        drop(handle);

        testing_logger::validate(|captured_logs| {
            assert_eq!(captured_logs.len(), 1);
            assert_eq!(captured_logs[0].body, "codes_index_delete");
            assert_eq!(captured_logs[0].level, log::Level::Trace);
        });

        Ok(())
    }

    #[test]
    #[cfg(feature = "experimental_index")]
    fn empty_index_constructor() -> Result<()> {
        let index =
            CodesIndex::new_from_keys(&vec!["shortName", "typeOfLevel", "level", "stepType"])?;

        let mut handle = CodesHandle::new_from_index(index)?;

        assert!(!handle.source.pointer.is_null());
        assert!(handle.current_message.is_none());

        let msg = handle.next()?;

        assert!(!msg.is_some());

        Ok(())
    }

    #[test]
    #[cfg(feature = "experimental_index")]
    fn multiple_drops_with_index() -> Result<()> {
        testing_logger::setup();
        {
            let keys = vec!["typeOfLevel", "level"];
            let index = CodesIndex::new_from_keys(&keys)?;
            let grib_path = Path::new("./data/iceland-levels.grib");
            let index = index
                .add_grib_file(grib_path)?
                .select("typeOfLevel", "isobaricInhPa")?
                .select("level", 600)?;

            let mut handle = CodesHandle::new_from_index(index)?;
            let _ref_msg = handle.next()?.context("no message")?;
            let clone_msg = _ref_msg.try_clone()?;
            let _oth_ref = handle.next()?.context("no message")?;

            let _nrst = clone_msg.codes_nearest()?;
            let _kiter = clone_msg.default_keys_iterator()?;
        }

        testing_logger::validate(|captured_logs| {
            assert_eq!(captured_logs.len(), 6);

            let expected_logs = vec![
                ("codes_handle_delete", log::Level::Trace),
                ("codes_keys_iterator_delete", log::Level::Trace),
                ("codes_grib_nearest_delete", log::Level::Trace),
                ("codes_handle_delete", log::Level::Trace),
                ("codes_handle_delete", log::Level::Trace),
                ("codes_index_delete", log::Level::Trace),
            ];

            captured_logs
                .iter()
                .zip(expected_logs)
                .for_each(|(clg, elg)| {
                    assert_eq!(clg.body, elg.0);
                    assert_eq!(clg.level, elg.1)
                });
        });

        Ok(())
    }
}
