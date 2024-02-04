mod iterator;
mod read;
mod write;

use log::warn;
use std::ptr::null_mut;

use crate::{
    codes_handle::KeyedMessage,
    intermediate_bindings::{codes_handle_clone, codes_handle_delete},
};

use super::KeysIteratorFlags;

impl Clone for KeyedMessage {
    ///Custom function to clone the `KeyedMessage`. This function comes with memory overhead.
    ///During clone iterator flags and namespace are not copied, and the iterator is reset.
    fn clone(&self) -> KeyedMessage {
        let new_handle =
            unsafe { codes_handle_clone(self.message_handle).expect("Cannot clone the message") };

        KeyedMessage {
            message_handle: new_handle,
        }
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
    use anyhow::Result;
    use std::path::Path;
    use testing_logger;

    #[test]
    fn message_clone_1() -> Result<()> {
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;
        let current_message = handle.next()?.unwrap();
        let cloned_message = current_message.clone();

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
        let current_message = handle.next()?.unwrap().clone();

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
