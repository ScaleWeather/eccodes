//!Main crate module containing definition of `CodesHandle`
//!and all associated functions and data structures

use crate::errors::CodesError;
use bytes::Bytes;
use eccodes_sys::{codes_handle, ProductKind_PRODUCT_GRIB};
use errno::errno;
use libc::{c_char, c_void, size_t, FILE};
use log::warn;
use std::{
    fs::{File, OpenOptions},
    os::unix::prelude::AsRawFd,
    path::Path,
    ptr::null_mut,
};

mod iterator;
mod keyed_message;

///Main structure used to operate on the GRIB file.
///It takes a full ownership of the accessed file.
///It can be constructed either using a file or a memory buffer.
#[derive(Debug)]
pub struct CodesHandle {
    file_handle: *mut codes_handle,
    data: DataContainer,
    file_pointer: *mut FILE,
    product_kind: ProductKind,
}

///Structure used to access keys inside the GRIB file message.
///All data (including data values) contained by the file can only be accessed
///through the message and keys.
#[derive(Hash, Debug)]
pub struct KeyedMessage {
    message_handle: *mut codes_handle,
    message_buffer: Vec<u8>,
}

///Enum to represent and contain all possible types of keys inside `KeyedMessage`.
///
///Messages inside GRIB files can contain arbitrary keys set by the file author.
///The type of a given key is only known at runtime (after being checked).
///There are several possible types of keys, which are represented by this enum
///and each variant contains the respective data type.
#[derive(Clone, Debug, PartialEq)]
pub enum Key {
    Float(f64),
    Int(i64),
    FloatArray(Vec<f64>),
    IntArray(Vec<i64>),
    Str(String),
}

#[derive(Debug)]
enum DataContainer {
    FileBytes(Bytes),
    FileBuffer(File),
}

///Enum representing the kind of product (file type) inside handled file.
///Used to indicate to ecCodes how it should decode/encode messages.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum ProductKind {
    GRIB = ProductKind_PRODUCT_GRIB as isize,
}

impl CodesHandle {
    ///The constructor that takes a [`path`](Path) to an existing file and
    ///a requested [`ProductKind`] and returns the [`CodesHandle`] object.
    ///
    ///## Example
    ///
    ///```
    ///# use eccodes::codes_handle::{ProductKind, CodesHandle};
    ///# use std::path::Path;
    ///#
    ///let file_path = Path::new("./data/iceland.grib");
    ///let product_kind = ProductKind::GRIB;
    ///
    ///let handle = CodesHandle::new_from_file(file_path, product_kind).unwrap();
    ///```
    ///
    ///The function opens the file as [`File`] and then utilises
    ///[`fdopen()`](https://man7.org/linux/man-pages/man3/fdopen.3.html) function
    ///to associate [`io::RawFd`](`std::os::unix::io::RawFd`) from [`File`]
    ///with a stream represented by [`libc::FILE`](https://docs.rs/libc/0.2.101/libc/enum.FILE.html) pointer.
    ///
    ///The constructor takes a [`path`](Path) as an argument instead of [`File`]
    ///to ensure that `fdopen()` uses the same mode as [`File`].
    ///The file descriptor does not take the ownership of a file, therefore the
    ///[`File`] is safely closed when it is dropped.
    ///
    ///## Errors
    ///Returns [`CodesError::CantOpenFile`] with [`io::Error`](std::io::Error)
    ///when the file cannot be opened.
    ///
    ///Returns [`CodesError::LibcNonZero`] with [`errno`](errno::Errno) information
    ///when the stream cannot be created from the file descriptor.
    ///
    ///Returns [`CodesError::Internal`] with error code
    ///when internal [`codes_handle`](eccodes_sys::codes_handle) cannot be created.
    ///
    ///Returns [`CodesError::NoMessages`] when there is no message of requested type
    ///in the provided file.
    pub fn new_from_file(file_path: &Path, product_kind: ProductKind) -> Result<Self, CodesError> {
        let file = OpenOptions::new().read(true).open(file_path)?;
        let file_pointer = open_with_fdopen(&file)?;

        let file_handle = null_mut();

        Ok(CodesHandle {
            data: (DataContainer::FileBuffer(file)),
            file_handle,
            file_pointer,
            product_kind,
        })
    }

    ///The constructor that takes data of file present in memory in [`Bytes`] format and
    ///a requested [`ProductKind`] and returns the [`CodesHandle`] object.
    ///
    ///## Example
    ///
    ///```
    ///# async fn run() {
    ///# use eccodes::codes_handle::{ProductKind, CodesHandle};
    ///#
    ///let product_kind = ProductKind::GRIB;
    ///let file_data =
    ///    reqwest::get("https://github.com/ScaleWeather/eccodes/blob/main/data/iceland.grib?raw=true")
    ///        .await
    ///        .unwrap()
    ///        .bytes()
    ///        .await
    ///        .unwrap();
    ///
    ///let handle = CodesHandle::new_from_memory(file_data, product_kind).unwrap();
    ///# }
    ///```
    ///
    ///The function associates the data in memory with a stream
    ///represented by [`libc::FILE`](https://docs.rs/libc/0.2.101/libc/enum.FILE.html) pointer
    ///using [`fmemopen()`](https://man7.org/linux/man-pages/man3/fmemopen.3.html) function.
    ///
    ///The constructor takes a full ownership of the data inside [`Bytes`],
    ///which is safely dropped during the [`CodesHandle`] drop.
    ///
    ///## Errors
    ///Returns [`CodesError::LibcNonZero`] with [`errno`](errno::Errno) information
    ///when the file stream cannot be created.
    ///
    ///Returns [`CodesError::Internal`] with error code
    ///when internal [`codes_handle`](eccodes_sys::codes_handle) cannot be created.
    ///
    ///Returns [`CodesError::NoMessages`] when there is no message of requested type
    ///in the provided file.
    pub fn new_from_memory(
        file_data: Bytes,
        product_kind: ProductKind,
    ) -> Result<Self, CodesError> {
        let file_pointer = open_with_fmemopen(&file_data)?;

        let file_handle = null_mut();

        Ok(CodesHandle {
            data: (DataContainer::FileBytes(file_data)),
            file_handle,
            file_pointer,
            product_kind,
        })
    }
}

fn open_with_fdopen(file: &File) -> Result<*mut FILE, CodesError> {
    let file_ptr;
    unsafe {
        file_ptr = libc::fdopen(file.as_raw_fd(), "r".as_ptr().cast::<c_char>());
    }

    if file_ptr.is_null() {
        let error_val = errno();
        let error_code = error_val.0;
        return Err(CodesError::LibcNonZero(error_code, error_val));
    }

    Ok(file_ptr)
}

fn open_with_fmemopen(file_data: &Bytes) -> Result<*mut FILE, CodesError> {
    let file_ptr;
    unsafe {
        file_ptr = libc::fmemopen(
            file_data.as_ptr() as *mut c_void,
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

impl Drop for CodesHandle {
    ///Executes the destructor for this type.
    ///This method calls `fclose()` from libc for graceful cleanup.
    ///
    ///Currently it is assumed that under normal circumstances this destructor never fails.
    ///However in some edge cases fclose can return non-zero code.
    ///In such case all pointers and file descriptors are safely deleted.
    ///However memory leaks can still occur.
    ///
    ///If any function called in the destructor returns an error warning will appear in log.
    ///If bugs occurs during `CodesHandle` drop please enable log output and post issue on [Github](https://github.com/ScaleWeather/eccodes).
    fn drop(&mut self) {
        //fclose() can fail in several different cases, however there is not much
        //that we can nor we should do about it. the promise of fclose() is that
        //the stream will be disassociated from the file after the call, therefore
        //use of stream after the call to fclose() is undefined behaviour, so we clear it
        let return_code;
        unsafe {
            return_code = libc::fclose(self.file_pointer);
        }

        if return_code != 0 {
            let error_val = errno();
            warn!(
                "fclose() returned an error and your file might not have been correctly saved.
                Error code: {}; Error message: {}",
                error_val.0, error_val
            );
        }

        self.file_pointer = null_mut();
    }
}

#[cfg(test)]
mod tests {
    use eccodes_sys::ProductKind_PRODUCT_GRIB;

    use crate::codes_handle::{CodesHandle, DataContainer, ProductKind};
    use std::path::Path;

    #[test]
    fn file_constructor() {
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let handle = CodesHandle::new_from_file(file_path, product_kind).unwrap();

        assert!(!handle.file_pointer.is_null());
        assert!(handle.file_handle.is_null());
        assert_eq!(handle.product_kind as u32, ProductKind_PRODUCT_GRIB as u32);

        let metadata = match &handle.data {
            DataContainer::FileBytes(_) => panic!(),
            DataContainer::FileBuffer(file) => file.metadata().unwrap(),
        };

        println!("{:?}", metadata);
    }

    #[tokio::test]
    async fn memory_constructor() {
        let product_kind = ProductKind::GRIB;
        let file_data = reqwest::get(
            "https://github.com/ScaleWeather/eccodes/blob/main/data/iceland.grib?raw=true",
        )
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();

        let handle = CodesHandle::new_from_memory(file_data, product_kind).unwrap();
        assert!(!handle.file_pointer.is_null());
        assert!(handle.file_handle.is_null());
        assert_eq!(handle.product_kind as u32, ProductKind_PRODUCT_GRIB as u32);

        match &handle.data {
            DataContainer::FileBytes(file) => assert!(!file.is_empty()),
            DataContainer::FileBuffer(_) => panic!(),
        };
    }
}
