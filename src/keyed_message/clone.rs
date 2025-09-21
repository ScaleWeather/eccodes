use crate::{
    AtomicMessage, CodesError, KeyedMessage, codes_handle::ThreadSafeHandle,
    intermediate_bindings::codes_handle_clone, keyed_message::ClonedMessage,
};

pub trait TryClone {
    /// Custom function to clone the `KeyedMessage` and `AtomicMessage`.
    ///
    /// **Be careful of the memory overhead!** ecCodes (when reading from file) defers reading the data into memory
    /// if possible. Simply creating `KeyedMessage` or even reading some keys will use only a little of memory.
    /// This function **will** read the whole message into the memory, which can be of a significant size for big grids.
    ///
    /// # Errors
    /// This function will return [`CodesInternal`](crate::errors::CodesInternal) if ecCodes fails to clone the message.
    fn try_clone(&self) -> Result<ClonedMessage, CodesError>;
}

impl TryClone for KeyedMessage<'_> {
    fn try_clone(&self) -> Result<ClonedMessage, CodesError> {
        Ok(ClonedMessage {
            message_handle: unsafe { codes_handle_clone(self.message_handle)? },
        })
    }
}

impl<S: ThreadSafeHandle> TryClone for AtomicMessage<S> {
    fn try_clone(&self) -> Result<ClonedMessage, CodesError> {
        Ok(ClonedMessage {
            message_handle: unsafe { codes_handle_clone(self.message_handle)? },
        })
    }
}
