//! Definition and constructors of `CodesHandle`
//! used for accessing GRIB files

#[cfg(feature = "experimental_index")]
use crate::{codes_index::CodesIndex, intermediate_bindings::codes_index_delete};
use crate::{pointer_guard, CodesError, KeyedMessage};
use bytes::Bytes;
use eccodes_sys::ProductKind_PRODUCT_GRIB;
use errno::errno;
use libc::{c_char, c_void, size_t, FILE};
use log::warn;
use std::{
    fmt::Debug,
    fs::{File, OpenOptions},
    os::unix::prelude::AsRawFd,
    path::Path,
    ptr::null_mut,
};

mod iterator;

/// This is an internal structure used to access provided file by `CodesHandle`.
/// It also allows to differentiate between `CodesHandle` created from file and from index.
/// It is not intended to be used directly by the user.
#[derive(Debug)]
pub struct GribFile {
    pointer: *mut FILE,
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
/// leading to bugs. Errors encountered during the destructor are logged with [`log`].
/// 
/// # `FallibleStreamingIterator`
/// 
/// This structure implements [`FallibleStreamingIterator`](crate::FallibleStreamingIterator) trait which allows to access GRIB messages.
/// 
/// To access GRIB messages the ecCodes library uses a method similar to a C-style iterator.
/// It digests the `* FILE` multiple times, each time returning the `*mut codes_handle`
/// to a message inside the file. The behavior of previous `*mut codes_handle` after next one is generated is undefined
/// and we assume here that it is unsafe to use "old" `*mut codes_handle`.
/// 
/// In Rust, such pattern is best represented by a streaming iterator which returns a reference to the message,
/// that is valid only until the next iteration. If you need to prolong the lifetime of the message, you can clone it.
/// Internal ecCodes functions can fail, necessitating the streaming iterator to be implemented with 
/// [`FallibleStreamingIterator`](crate::FallibleStreamingIterator) trait.
/// 
/// As of `0.10` release, none of the available streaming iterator crates utilises already stabilized GATs.
/// This unfortunately significantly limits the number of methods available for `CodesHandle` iterator.
/// Therefore the probably most versatile way to iterate over the messages is to use `while let` loop.
/// 
/// ```
/// use eccodes::{ProductKind, CodesHandle, KeyType};
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
///     let key = message.read_key("name")?;
/// 
///     if let KeyType::Str(name) = key.value {
///         println!("{:?}", name);    
///     }
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
pub struct CodesHandle<SOURCE: Debug + SpecialDrop> {
    _data: DataContainer,
    source: SOURCE,
    product_kind: ProductKind,
    unsafe_message: KeyedMessage,
}

#[derive(Debug)]
enum DataContainer {
    FileBytes(Bytes),
    FileBuffer(File),
    #[cfg(feature = "experimental_index")]
    Empty(),
}

///Enum representing the kind of product (file type) inside handled file.
///Used to indicate to ecCodes how it should decode/encode messages.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum ProductKind {
    GRIB = ProductKind_PRODUCT_GRIB as isize,
}

impl CodesHandle<GribFile> {
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
    ///when internal [`codes_handle`](eccodes_sys::codes_handle) cannot be created.
    pub fn new_from_file(file_path: &Path, product_kind: ProductKind) -> Result<Self, CodesError> {
        let file = OpenOptions::new().read(true).open(file_path)?;
        let file_pointer = open_with_fdopen(&file)?;

        Ok(CodesHandle {
            _data: (DataContainer::FileBuffer(file)),
            source: GribFile {
                pointer: file_pointer,
            },
            product_kind,
            unsafe_message: KeyedMessage {
                message_handle: null_mut(),
            },
        })
    }

    ///Opens data in provided [`Bytes`] buffer as selected [`ProductKind`] and contructs `CodesHandle`.
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
    ///        .await?;
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
    ///The constructor takes full ownership of the data inside [`Bytes`],
    ///which is safely dropped during the [`CodesHandle`] drop.
    ///
    ///## Errors
    ///Returns [`CodesError::LibcNonZero`] with [`errno`](errno::Errno) information
    ///when the file stream cannot be created.
    ///
    ///Returns [`CodesError::Internal`] with error code
    ///when internal [`codes_handle`](eccodes_sys::codes_handle) cannot be created.
    pub fn new_from_memory(
        file_data: Bytes,
        product_kind: ProductKind,
    ) -> Result<Self, CodesError> {
        let file_pointer = open_with_fmemopen(&file_data)?;

        Ok(CodesHandle {
            _data: (DataContainer::FileBytes(file_data)),
            source: GribFile {
                pointer: file_pointer,
            },
            product_kind,
            unsafe_message: KeyedMessage {
                message_handle: null_mut(),
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
    /// when internal [`codes_handle`](eccodes_sys::codes_handle) cannot be created.
    pub fn new_from_index(
        index: CodesIndex,
    ) -> Result<Self, CodesError> {
        let new_handle = CodesHandle {
            _data: DataContainer::Empty(), //unused, index owns data
            source: index,
            product_kind: ProductKind::GRIB,
            unsafe_message: KeyedMessage {
                message_handle: null_mut(),
            },
        };

        Ok(new_handle)
    }
}

fn open_with_fdopen(file: &File) -> Result<*mut FILE, CodesError> {
    let file_ptr = unsafe { libc::fdopen(file.as_raw_fd(), "r".as_ptr().cast::<c_char>()) };

    if file_ptr.is_null() {
        let error_val = errno();
        let error_code = error_val.0;
        return Err(CodesError::LibcNonZero(error_code, error_val));
    }

    Ok(file_ptr)
}

fn open_with_fmemopen(file_data: &Bytes) -> Result<*mut FILE, CodesError> {
    let file_data_ptr = file_data.as_ptr() as *mut c_void;
    pointer_guard::non_null!(file_data_ptr);

    let file_ptr;
    unsafe {
        file_ptr = libc::fmemopen(
            file_data_ptr,
            file_data.len() as size_t,
            "r".as_ptr().cast::<c_char>(),
        );
    }

    if file_ptr.is_null() {
        let error_val = errno();
        let error_code = error_val.0;
        return Err(CodesError::LibcNonZero(error_code, error_val));
    }

    Ok(file_ptr)
}

// This trait is neccessary because (1) drop in GribFile/IndexFile cannot
// be called directly as source cannot be moved out of shared reference
// and (2) Drop drops fields in arbitrary order leading to fclose() failing
#[doc(hidden)]
pub trait SpecialDrop {
    fn spec_drop(&mut self);
}

#[doc(hidden)]
impl SpecialDrop for GribFile {
    fn spec_drop(&mut self) {
        //fclose() can fail in several different cases, however there is not much
        //that we can nor we should do about it. the promise of fclose() is that
        //the stream will be disassociated from the file after the call, therefore
        //use of stream after the call to fclose() is undefined behaviour, so we clear it
        let return_code;
        unsafe {
            if !self.pointer.is_null() {
                return_code = libc::fclose(self.pointer);
                if return_code != 0 {
                    let error_val = errno();
                    warn!(
                "fclose() returned an error and your file might not have been correctly saved.
                Error code: {}; Error message: {}",
                error_val.0, error_val
            );
                }
            }
        }

        self.pointer = null_mut();
    }
}

#[doc(hidden)]
#[cfg(feature = "experimental_index")]
impl SpecialDrop for CodesIndex {
    fn spec_drop(&mut self) {
        unsafe {
            codes_index_delete(self.pointer);
        }

        self.pointer = null_mut();
    }
}

#[doc(hidden)]
impl<S: Debug + SpecialDrop> Drop for CodesHandle<S> {
    /// Executes the destructor for this type.
    /// 
    /// Currently it is assumed that under normal circumstances this destructor never fails.
    /// However in some edge cases fclose can return non-zero code.
    /// In such case all pointers and file descriptors are safely deleted.
    /// However memory leaks can still occur.
    /// 
    /// If any function called in the destructor returns an error warning will appear in log.
    /// If bugs occurs during `CodesHandle` drop please enable log output and post issue on [Github](https://github.com/ScaleWeather/eccodes).
    fn drop(&mut self) {
        self.source.spec_drop();
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use eccodes_sys::ProductKind_PRODUCT_GRIB;

    use crate::codes_handle::{CodesHandle, DataContainer, ProductKind};
    #[cfg(feature = "experimental_index")]
    use crate::codes_index::{CodesIndex, Select};
    use log::Level;
    use std::path::Path;

    #[test]
    fn file_constructor() -> Result<()> {
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let handle = CodesHandle::new_from_file(file_path, product_kind)?;

        assert!(!handle.source.pointer.is_null());
        assert!(handle.unsafe_message.message_handle.is_null());
        assert_eq!(handle.product_kind as u32, { ProductKind_PRODUCT_GRIB });

        match &handle._data {
            DataContainer::FileBuffer(file) => file.metadata()?,
            _ => panic!(),
        };

        Ok(())
    }

    #[tokio::test]
    async fn memory_constructor() -> Result<()> {
        let product_kind = ProductKind::GRIB;
        let file_data = reqwest::get(
            "https://github.com/ScaleWeather/eccodes/blob/main/data/iceland.grib?raw=true",
        )
        .await?
        .bytes()
        .await?;

        let handle = CodesHandle::new_from_memory(file_data, product_kind)?;
        assert!(!handle.source.pointer.is_null());
        assert!(handle.unsafe_message.message_handle.is_null());
        assert_eq!(handle.product_kind as u32, { ProductKind_PRODUCT_GRIB });

        match &handle._data {
            DataContainer::FileBytes(file) => assert!(!file.is_empty()),
            _ => panic!(),
        };

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
        assert!(handle.unsafe_message.message_handle.is_null());

        Ok(())
    }

    #[tokio::test]
    async fn codes_handle_drop() -> Result<()> {
        testing_logger::setup();

        {
            let file_path = Path::new("./data/iceland-surface.grib");
            let product_kind = ProductKind::GRIB;

            let handle = CodesHandle::new_from_file(file_path, product_kind)?;
            drop(handle);

            testing_logger::validate(|captured_logs| {
                assert_eq!(captured_logs.len(), 0);
            });
        }

        {
            let product_kind = ProductKind::GRIB;
            let file_data = reqwest::get(
                "https://github.com/ScaleWeather/eccodes/blob/main/data/iceland.grib?raw=true",
            )
            .await?
            .bytes()
            .await?;

            let handle = CodesHandle::new_from_memory(file_data, product_kind)?;
            drop(handle);

            //logs from Reqwest are expected
            testing_logger::validate(|captured_logs| {
                for log in captured_logs {
                    assert_ne!(log.level, Level::Warn);
                    assert_ne!(log.level, Level::Error);
                }
            });
        }

        Ok(())
    }

    #[test]
    #[cfg(feature = "experimental_index")]
    fn empty_index_constructor() -> Result<()> {
        use fallible_streaming_iterator::FallibleStreamingIterator;

        let index =
            CodesIndex::new_from_keys(&vec!["shortName", "typeOfLevel", "level", "stepType"])?;

        let mut handle = CodesHandle::new_from_index(index)?;

        assert!(!handle.source.pointer.is_null());
        assert!(handle.unsafe_message.message_handle.is_null());

        let msg = handle.next()?;

        assert!(!msg.is_some());

        Ok(())
    }
}
