#![allow(unused)]

use std::sync::Arc;

use eccodes_sys::codes_handle;
use fallible_iterator::FallibleIterator;

use crate::{CodesError, CodesHandle, codes_handle::ThreadSafeHandle};

#[derive(Debug)]
pub struct AtomicMessageGenerator<S: ThreadSafeHandle> {
    codes_handle: Arc<CodesHandle<S>>,
}
impl<S: ThreadSafeHandle> CodesHandle<S> {
    pub fn atomic_message_generator(self) -> AtomicMessageGenerator<S> {
        AtomicMessageGenerator {
            codes_handle: Arc::new(self),
        }
    }
}

impl<S: ThreadSafeHandle> FallibleIterator for AtomicMessageGenerator<S> {
    type Item = AtomicMessage<S>;

    type Error = CodesError;

    fn next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        let new_eccodes_handle = self.codes_handle.source.gen_codes_handle()?;

        if new_eccodes_handle.is_null() {
            Ok(None)
        } else {
            Ok(Some(AtomicMessage {
                _parent: self.codes_handle.clone(),
                pointer: new_eccodes_handle,
            }))
        }
    }
}

#[derive(Debug)]
pub struct AtomicMessage<S: ThreadSafeHandle> {
    _parent: Arc<CodesHandle<S>>,
    pointer: *mut codes_handle,
}
unsafe impl<S: ThreadSafeHandle> Send for AtomicMessage<S> {}
unsafe impl<S: ThreadSafeHandle> Sync for AtomicMessage<S> {}

#[cfg(test)]
mod tests {
    use std::{
        path::Path,
        sync::{Arc, Barrier},
    };

    use anyhow::{Context, Result};
    use fallible_iterator::FallibleIterator;

    use crate::{CodesHandle, ProductKind};

    #[test]
    fn atomic_thread_safety() -> Result<()> {
        let file_path = Path::new("./data/iceland-levels.grib");
        let product_kind = ProductKind::GRIB;

        let handle = CodesHandle::new_from_file(file_path, product_kind)?;
        let mut mgen = handle.atomic_message_generator();
        // let _ = handle.atomic_message_generator(); <- not allowed due to ownership

        let barrier = Arc::new(Barrier::new(10));

        let mut v = vec![];

        for _ in 0..10 {
            let msg = Arc::new(mgen.next()?.context("No more messages")?);
            let b = barrier.clone();

            let t = std::thread::spawn(move || {
                for _ in 0..1000 {
                    b.wait();
                    let _ = unsafe {
                        crate::intermediate_bindings::codes_get_size(msg.pointer, "shortName")
                            .unwrap()
                    };
                }
            });

            v.push(t);
        }

        v.into_iter().for_each(|th| th.join().unwrap());

        Ok(())
    }
}
