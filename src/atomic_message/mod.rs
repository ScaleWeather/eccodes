mod read;

use std::sync::Arc;

use eccodes_sys::codes_handle;

use crate::{CodesHandle, codes_handle::ThreadSafeHandle};

pub use read::{AtomicKeyRead};

/// Because standard `KeyedMessage` is not Copy or Clone it can provide access methods without
/// requiring `&mut self`. As `AtomicMessage` implements `Send + Sync` this exclusive method access is not
/// guaranteed with just `&self`. `AtomicMessage` also implements a minimal subset of functionalities
/// to limit the risk of some internal ecCodes functions not being thread-safe.
///
/// Right now `AtomicMessage` is also not clonable
#[derive(Debug)]
pub struct AtomicMessage<S: ThreadSafeHandle> {
    pub(crate) _parent: Arc<CodesHandle<S>>,
    pub(crate) message_handle: *mut codes_handle,
}

unsafe impl<S: ThreadSafeHandle> Send for AtomicMessage<S> {}
unsafe impl<S: ThreadSafeHandle> Sync for AtomicMessage<S> {}
