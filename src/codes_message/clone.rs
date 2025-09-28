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
    fn try_clone(&self) -> Result<BufMessage, CodesError> {
        Ok(BufMessage {
            _parent: (),
            message_handle: unsafe { codes_handle_clone(self.message_handle)? },
        })
    }
}
