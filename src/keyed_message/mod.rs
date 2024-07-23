//! Definition of `KeyedMessage` and its associated functions
//! used for reading and writing data of given variable from GRIB file

mod read;
mod write;

use eccodes_sys::codes_handle;
use log::warn;
use std::ptr::null_mut;

use crate::{
    intermediate_bindings::{codes_handle_clone, codes_handle_delete},
    CodesError,
};

/// Structure that provides access to the data contained in the GRIB file, which directly corresponds to the message in the GRIB file
///
/// **Usage examples are provided in documentation of each method.**
///
/// You can think about the message as a container of data corresponding to a single variable
/// at given date, time and level. In ecCodes the message is represented as a collection of unique
/// key-value pairs. Each [`Key`] in the message has a unique name and a value of type [`KeyType`].
///
/// You can read a `Key` directly using [`read_key()`](KeyedMessage::read_key()) or iterate over
/// all keys using [`KeysIterator`](crate::KeysIterator). You can also modify the message using
/// [`write_key()`](KeyedMessage::write_key()). As of `0.10`, this crate can successfully read all keys
/// from ERA5 and GFS files.
///
/// If you are interested only in getting data values from the message you can use
/// [`to_ndarray()`](KeyedMessage::to_ndarray) from the [`message_ndarray`](crate::message_ndarray) module.
///
/// Some of the useful keys are: `validityDate`, `validityTime`, `level`, `typeOfLevel`, `shortName`, `units` and `values`.
///
/// Note that names, types and availability of some keys can vary between platforms and ecCodes versions. You should test
/// your code whenever changing the environment.
///
/// [`CodesNearest`](crate::CodesNearest) can be used to find nearest gridpoints for given coordinates in the `KeyedMessage`.
///
/// Most of `KeyedMessage` methods (except for writing) can be used directly with `&KeyedMessage`
/// returned by `CodesHandle` iterator, which provides the best performance.
/// When mutable access or longer liftime is needed the message can be cloned with [`try_clone`](KeyedMessage::try_clone)
/// Note that cloning comes with a performance and memory overhead.
/// You should take care that your system has enough memory before cloning.
///
/// Destructor for this structure does not panic, but some internal functions may rarely fail
/// leading to bugs. Errors encountered in desctructor the are logged with [`log`].
#[derive(Hash, Debug)]
pub struct KeyedMessage {
    pub(crate) message_handle: *mut codes_handle,
}

/// Structure representing a single key in the `KeyedMessage`
#[derive(Clone, Debug, PartialEq)]
pub struct DynamicKey {
    #[allow(missing_docs)]
    pub name: String,
    #[allow(missing_docs)]
    pub value: DynamicKeyType,
}

/// Enum representing the value of [`Key`] from the `KeyedMessage`
///
/// Messages inside GRIB files can contain keys of arbitrary types, which are known only at runtime (after being checked).
/// ecCodes can return several different types of key, which are represented by this enum
/// and each variant contains the respective data type.
#[derive(Clone, Debug, PartialEq)]
pub enum DynamicKeyType {
    #[allow(missing_docs)]
    Float(f64),
    #[allow(missing_docs)]
    Int(i64),
    #[allow(missing_docs)]
    FloatArray(Vec<f64>),
    #[allow(missing_docs)]
    IntArray(Vec<i64>),
    #[allow(missing_docs)]
    Str(String),
    #[allow(missing_docs)]
    Bytes(Vec<u8>),
}

impl KeyedMessage {
    /// Custom function to clone the `KeyedMessage`. This function comes with memory overhead.
    ///
    /// # Errors
    /// This function will return [`CodesInternal`](crate::errors::CodesInternal) if ecCodes fails to clone the message.
    pub fn try_clone(&self) -> Result<Self, CodesError> {
        let new_handle = unsafe { codes_handle_clone(self.message_handle)? };

        Ok(Self {
            message_handle: new_handle,
        })
    }
}

#[doc(hidden)]
impl Drop for KeyedMessage {
    /// Executes the destructor for this type.
    /// This method calls destructor functions from ecCodes library.
    /// In some edge cases these functions can return non-zero code.
    /// In such case all pointers and file descriptors are safely deleted.
    /// However memory leaks can still occur.
    ///
    /// If any function called in the destructor returns an error warning will appear in log.
    /// If bugs occur during `CodesHandle` drop please enable log output and post issue on [Github](https://github.com/ScaleWeather/eccodes).
    ///
    /// Technical note: delete functions in ecCodes can only fail with [`CodesInternalError`](crate::errors::CodesInternal::CodesInternalError)
    /// when other functions corrupt the inner memory of pointer, in that case memory leak is possible.
    /// In case of corrupt pointer segmentation fault will occur.
    /// The pointers are cleared at the end of drop as they are not functional regardless of result of delete functions.
    fn drop(&mut self) {
        unsafe {
            codes_handle_delete(self.message_handle).unwrap_or_else(|error| {
                warn!("codes_handle_delete() returned an error: {:?}", &error);
            });
        }

        self.message_handle = null_mut();
    }
}

#[cfg(test)]
mod tests {
    use crate::codes_handle::{CodesHandle, ProductKind};
    use crate::FallibleStreamingIterator;
    use anyhow::{Context, Result};
    use std::path::Path;
    use testing_logger;

    #[test]
    fn check_docs_keys() -> Result<()> {
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;
        let current_message = handle.next()?.context("Message not some")?;

        let _ = current_message.read_key_dynamic("validityDate")?;
        let _ = current_message.read_key_dynamic("validityTime")?;
        let _ = current_message.read_key_dynamic("level")?;
        let _ = current_message.read_key_dynamic("shortName")?;
        let _ = current_message.read_key_dynamic("units")?;
        let _ = current_message.read_key_dynamic("values")?;
        let _ = current_message.read_key_dynamic("typeOfLevel")?;

        Ok(())
    }

    #[test]
    fn message_clone_1() -> Result<()> {
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;
        let current_message = handle.next()?.context("Message not some")?;
        let cloned_message = current_message.try_clone()?;

        assert_ne!(
            current_message.message_handle,
            cloned_message.message_handle
        );

        Ok(())
    }

    #[test]
    fn message_clone_2() -> Result<()> {
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;
        let msg = handle.next()?.context("Message not some")?.try_clone()?;
        let _ = handle.next()?;

        drop(handle);

        let _ = msg.read_key_dynamic("dataDate")?;
        let _ = msg.read_key_dynamic("jDirectionIncrementInDegrees")?;
        let _ = msg.read_key_dynamic("values")?;
        let _ = msg.read_key_dynamic("name")?;
        let _ = msg.read_key_dynamic("section1Padding")?;
        let _ = msg.read_key_dynamic("experimentVersionNumber")?;

        Ok(())
    }

    #[test]
    fn message_drop() -> Result<()> {
        testing_logger::setup();
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;
        let current_message = handle.next()?.context("Message not some")?.try_clone()?;

        let _kiter = current_message.default_keys_iterator()?;
        let _niter = current_message.codes_nearest()?;

        drop(handle);
        drop(_kiter);
        drop(_niter);
        drop(current_message);

        testing_logger::validate(|captured_logs| {
            assert_eq!(captured_logs.len(), 0);
        });

        Ok(())
    }
}
