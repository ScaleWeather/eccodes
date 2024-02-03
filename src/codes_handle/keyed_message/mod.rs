mod iterator;
mod nearest;
mod read;
mod write;

use log::warn;
use std::ptr::null_mut;

use crate::{
    codes_handle::KeyedMessage,
    errors::CodesError,
    intermediate_bindings::{
        codes_grib_nearest_new, codes_handle_clone, codes_handle_delete, codes_keys_iterator_delete,
    },
};

use super::{CodesNearest, KeysIteratorFlags};

impl KeyedMessage {
    pub fn codes_nearest(&self) -> Result<CodesNearest, CodesError> {
        let nearest_handle = unsafe { codes_grib_nearest_new(self.message_handle)? };

        Ok(CodesNearest {
            nearest_handle,
            parent_message: self,
        })
    }
}

impl Clone for KeyedMessage {
    ///Custom function to clone the `KeyedMessage`. This function comes with memory overhead.
    ///During clone iterator flags and namespace are not copied, and the iterator is reset.
    fn clone(&self) -> KeyedMessage {
        let new_handle =
            unsafe { codes_handle_clone(self.message_handle).expect("Cannot clone the message") };

        KeyedMessage {
            message_handle: new_handle,
            iterator_flags: None,
            iterator_namespace: None,
            keys_iterator: None,
            keys_iterator_next_item_exists: false,
            nearest_handle: None,
        }
    }
}

impl Drop for KeyedMessage {
    ///Executes the destructor for this type.
    ///This method calls `codes_handle_delete()`, `codes_keys_iterator_delete()`
    ///`codes_grib_nearest_delete()` from ecCodes for graceful cleanup.
    ///However in some edge cases ecCodes can return non-zero code.
    ///In such case all pointers and file descriptors are safely deleted.
    ///However memory leaks can still occur.
    ///
    ///If any function called in the destructor returns an error warning will appear in log.
    ///If bugs occurs during `CodesHandle` drop please enable log output and post issue on [Github](https://github.com/ScaleWeather/eccodes).
    ///
    ///Technical note: delete functions in ecCodes can only fail with [`CodesInternalError`](crate::errors::CodesInternal::CodesInternalError)
    ///when other functions corrupt the inner memory of pointer, in that case memory leak is possible.
    ///In case of corrupt pointer segmentation fault will occur.
    ///The pointers are cleared at the end of drop as they are not functional despite the result of delete functions.
    fn drop(&mut self) {
        if let Some(kiter) = self.keys_iterator {
            unsafe {
                codes_keys_iterator_delete(kiter).unwrap_or_else(|error| {
                    warn!(
                        "codes_keys_iterator_delete() returned an error: {:?}",
                        &error
                    );
                });
            }
        }

        self.keys_iterator = Some(null_mut());

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
    use crate::{FallibleIterator, FallibleStreamingIterator};
    use anyhow::Result;
    use std::path::Path;
    use testing_logger;

    #[test]
    fn key_clone() -> Result<()> {
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;
        let current_message = handle.next()?.unwrap();
        let cloned_message = current_message.clone();

        assert_ne!(
            current_message.message_handle,
            cloned_message.message_handle
        );
        assert!(cloned_message.iterator_flags.is_none());
        assert!(cloned_message.iterator_namespace.is_none());
        assert!(cloned_message.keys_iterator.is_none());
        assert!(!cloned_message.keys_iterator_next_item_exists);

        Ok(())
    }

    #[test]
    fn message_clone() -> Result<()> {
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;
        let msg = handle.next()?.unwrap().clone();
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
        let mut current_message = handle.next()?.unwrap().clone();

        let _key = current_message.next()?.unwrap();

        drop(current_message);

        testing_logger::validate(|captured_logs| {
            assert_eq!(captured_logs.len(), 0);
        });

        Ok(())
    }
}
