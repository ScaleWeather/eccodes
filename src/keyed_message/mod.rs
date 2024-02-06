//! Definition of `KeyedMessage` and its associated functions
//! used for reading and writing data of given variable from GRIB file

mod read;
mod write;

use eccodes_sys::codes_handle;
use log::warn;
use std::ptr::null_mut;

use crate::{intermediate_bindings::{codes_handle_clone, codes_handle_delete}, CodesError};

///Structure used to access keys inside the GRIB file message.
///All data (including data values) contained by the file can only be accessed
///through the message and keys.
///
///The structure implements `Clone` trait which comes with a memory overhead.
///You should take care that your system has enough memory before cloning `KeyedMessage`.
///
///Keys inside the message can be accessed directly with [`read_key()`](KeyedMessage::read_key())
///function or using [`FallibleIterator`](KeyedMessage#impl-FallibleIterator).
///The function [`find_nearest()`](crate::codes_nearest::CodesNearest::find_nearest()) allows to get the values of four nearest gridpoints
///to requested coordinates.
#[derive(Hash, Debug)]
pub struct KeyedMessage {
    pub(crate) message_handle: *mut codes_handle,
}

///Structure representing a single key from the `KeyedMessage`.
#[derive(Clone, Debug, PartialEq)]
pub struct Key {
    pub name: String,
    pub value: KeyType,
}

///Enum to represent and contain all possible types of keys inside `KeyedMessage`.
///
///Messages inside GRIB files can contain arbitrary keys set by the file author.
///The type of a given key is only known at runtime (after being checked).
///There are several possible types of keys, which are represented by this enum
///and each variant contains the respective data type.
#[derive(Clone, Debug, PartialEq)]
pub enum KeyType {
    Float(f64),
    Int(i64),
    FloatArray(Vec<f64>),
    IntArray(Vec<i64>),
    Str(String),
    Bytes(Vec<u8>),
}

impl KeyedMessage {
    ///Custom function to clone the `KeyedMessage`. This function comes with memory overhead.
    /// 
    /// # Errors
    /// This function will return [`CodesInternal`](crate::errors::CodesInternal) if ecCodes fails to clone the message.
    pub fn try_clone(&self) -> Result<KeyedMessage, CodesError> {
        let new_handle =
            unsafe { codes_handle_clone(self.message_handle)? };

        Ok(KeyedMessage {
            message_handle: new_handle,
        })
    }
}

impl Drop for KeyedMessage {
    ///Executes the destructor for this type.
    ///This method calls destructor functions from ecCodes library.
    ///In some edge cases these functions can return non-zero code.
    ///In such case all pointers and file descriptors are safely deleted.
    ///However memory leaks can still occur.
    ///
    ///If any function called in the destructor returns an error warning will appear in log.
    ///If bugs occur during `CodesHandle` drop please enable log output and post issue on [Github](https://github.com/ScaleWeather/eccodes).
    ///
    ///Technical note: delete functions in ecCodes can only fail with [`CodesInternalError`](crate::errors::CodesInternal::CodesInternalError)
    ///when other functions corrupt the inner memory of pointer, in that case memory leak is possible.
    ///In case of corrupt pointer segmentation fault will occur.
    ///The pointers are cleared at the end of drop as they are not functional regardless of result of delete functions.
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

        let _ = msg.read_key("dataDate")?;
        let _ = msg.read_key("jDirectionIncrementInDegrees")?;
        let _ = msg.read_key("values")?;
        let _ = msg.read_key("name")?;
        let _ = msg.read_key("section1Padding")?;
        let _ = msg.read_key("experimentVersionNumber")?;

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
