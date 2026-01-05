use crate::{
    BufMessage, CodesError, codes_message::CodesMessage, intermediate_bindings::codes_handle_clone,
};
use std::fmt::Debug;

impl<P: Debug> CodesMessage<P> {
    /// Custom function to clone the `KeyedMessage` and `AtomicMessage`.
    ///
    /// **Be careful of the memory overhead!** ecCodes (when reading from file) defers reading the data into memory
    /// if possible. Simply creating `KeyedMessage` or even reading some keys will use only a little of memory.
    /// This function **will** read the whole message into the memory, which can be of a significant size for big grids.
    ///
    /// # Errors
    /// This function will return [`CodesInternal`](crate::errors::CodesInternal) if ecCodes fails to clone the message.
    pub fn try_clone(&self) -> Result<BufMessage, CodesError> {
        Ok(BufMessage::new(unsafe {
            codes_handle_clone(self.message_handle)?
        }))
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use anyhow::{Context, Result};
    use fallible_iterator::FallibleIterator;

    use crate::{CodesFile, ProductKind};

    #[test]
    fn check_clone_safety() -> Result<()> {
        let file_path = Path::new("./data/iceland-levels.grib");
        let product_kind = ProductKind::GRIB;
        let mut handle = CodesFile::new_from_file(file_path, product_kind)?;

        let msg1 = handle
            .ref_message_iter()
            .next()?
            .context("Message not some")?;
        let key1 = msg1.read_key_dynamic("typeOfLevel")?;

        let msg_clone = msg1.try_clone()?;
        drop(msg1);
        drop(handle);
        let key1_clone = msg_clone.read_key_dynamic("typeOfLevel")?;
        assert_eq!(key1, key1_clone);

        Ok(())
    }
}
