//! Definition of `KeysIterator` used for iterating through keys in `KeyedMessage`

use eccodes_sys::codes_keys_iterator;
use fallible_iterator::FallibleIterator;
use log::warn;
use std::{marker::PhantomData, ptr::null_mut};

use crate::{
    errors::CodesError,
    intermediate_bindings::{
        codes_keys_iterator_delete, codes_keys_iterator_get_name, codes_keys_iterator_new,
        codes_keys_iterator_next,
    },
    KeyedMessage,
};

/// Structure to iterate through key names in [`KeyedMessage`].
///
/// Mainly useful to discover what keys are present inside the message.
///
/// Implements [`FallibleIterator`] providing similar functionality to classic `Iterator`.
/// `FallibleIterator` is used because internal ecCodes functions can return internal error in some edge-cases.
/// The usage of `FallibleIterator` is sligthly different than usage of `Iterator`,
/// check the documentation for more details.
///
/// To discover key contents you need to [`read_key`](crate::KeyRead::read_key) with name given by the iterator.
///
/// ## Example
///
/// ```
///  use eccodes::{ProductKind, CodesHandle, KeyedMessage, KeysIteratorFlags};
///  # use std::path::Path;
///  # use anyhow::Context;
///  use eccodes::{FallibleIterator, FallibleStreamingIterator};
///  #
///  # fn main() -> anyhow::Result<()> {
///  #
///  let file_path = Path::new("./data/iceland.grib");
///  let product_kind = ProductKind::GRIB;
///  
///  let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;
///  let current_message = handle.next()?.context("no message")?;
///  
///  let mut keys_iter = current_message.default_keys_iterator()?;
///  
///  while let Some(key_name) = keys_iter.next()? {
///      println!("{key_name}");
///  }
///  # Ok(())
///  # }
/// ```
///
/// ## Errors
///
/// The `next()` method will return [`CodesInternal`](crate::errors::CodesInternal)
/// when internal ecCodes function returns non-zero code.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct KeysIterator<'a> {
    parent_message: PhantomData<&'a KeyedMessage>,
    iterator_handle: *mut codes_keys_iterator,
    next_item_exists: bool,
}

/// Flags to specify the subset of keys to iterate over in
/// `KeysIterator`. Flags can be combined as needed.
#[allow(clippy::module_name_repetitions)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum KeysIteratorFlags {
    /// Iterate over all keys
    AllKeys = eccodes_sys::CODES_KEYS_ITERATOR_ALL_KEYS as isize,
    /// Iterate only over dump keys
    DumpOnly = eccodes_sys::CODES_KEYS_ITERATOR_DUMP_ONLY as isize,
    /// Exclude coded keys from iteration
    SkipCoded = eccodes_sys::CODES_KEYS_ITERATOR_SKIP_CODED as isize,
    /// Exclude computed keys from iteration
    SkipComputed = eccodes_sys::CODES_KEYS_ITERATOR_SKIP_COMPUTED as isize,
    /// Exclude function keys from iteration
    SkipFunction = eccodes_sys::CODES_KEYS_ITERATOR_SKIP_FUNCTION as isize,
    /// Exclude optional keys from iteration
    SkipOptional = eccodes_sys::CODES_KEYS_ITERATOR_SKIP_OPTIONAL as isize,
    /// Exclude read-only keys from iteration
    SkipReadOnly = eccodes_sys::CODES_KEYS_ITERATOR_SKIP_READ_ONLY as isize,
    /// Exclude duplicate keys from iteration
    SkipDuplicates = eccodes_sys::CODES_KEYS_ITERATOR_SKIP_DUPLICATES as isize,
    /// Exclude file edition specific keys from iteration
    SkipEditionSpecific = eccodes_sys::CODES_KEYS_ITERATOR_SKIP_EDITION_SPECIFIC as isize,
}

impl KeyedMessage {
    /// Creates new [`KeysIterator`] for the message with specified flags and namespace.
    ///
    /// The flags are set by providing any combination of [`KeysIteratorFlags`]
    /// inside a slice. Check the documentation for the details of each flag meaning.
    ///
    /// Namespace is set simply as string, eg. `"ls"`, `"time"`, `"parameter"`, `"geography"`, `"statistics"`.
    /// Invalid namespace will result in empty iterator.
    ///
    /// # Example
    ///
    /// ```
    ///  use eccodes::{ProductKind, CodesHandle, KeyedMessage, KeysIteratorFlags};
    ///  # use std::path::Path;
    ///  # use anyhow::Context;
    ///  use eccodes::{FallibleIterator, FallibleStreamingIterator};
    ///  #
    ///  # fn main() -> anyhow::Result<()> {
    ///  #
    ///  let file_path = Path::new("./data/iceland.grib");
    ///  let product_kind = ProductKind::GRIB;
    ///  
    ///  let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;
    ///  let current_message = handle.next()?.context("no message")?;
    ///  
    ///  let flags = [
    ///      KeysIteratorFlags::AllKeys,
    ///      KeysIteratorFlags::SkipOptional,
    ///      KeysIteratorFlags::SkipReadOnly,
    ///      KeysIteratorFlags::SkipDuplicates,
    ///  ];
    ///  
    ///  let namespace = "geography";
    ///  
    ///  let mut keys_iter = current_message.new_keys_iterator(&flags, namespace)?;
    ///  
    ///  while let Some(key_name) = keys_iter.next()? {
    ///      println!("{key_name}");
    ///  }
    ///  # Ok(())
    ///  # }
    /// ```
    ///
    /// # Errors
    ///
    /// This function returns [`CodesInternal`](crate::errors::CodesInternal) when
    /// internal ecCodes function returns non-zero code.
    pub fn new_keys_iterator(
        &self,
        flags: &[KeysIteratorFlags],
        namespace: &str,
    ) -> Result<KeysIterator, CodesError> {
        let flags = flags.iter().map(|f| *f as u32).sum();

        let iterator_handle =
            unsafe { codes_keys_iterator_new(self.message_handle, flags, namespace)? };
        let next_item_exists = unsafe { codes_keys_iterator_next(iterator_handle)? };

        Ok(KeysIterator {
            parent_message: PhantomData,
            iterator_handle,
            next_item_exists,
        })
    }

    /// Same as [`new_keys_iterator()`](KeyedMessage::new_keys_iterator) but with default
    /// parameters: [`AllKeys`](KeysIteratorFlags::AllKeys) flag and `""` namespace,
    /// yeilding iterator over all keys in the message.
    ///
    /// # Errors
    ///
    /// This function returns [`CodesInternal`](crate::errors::CodesInternal) when
    /// internal ecCodes function returns non-zero code.
    pub fn default_keys_iterator(&self) -> Result<KeysIterator, CodesError> {
        let iterator_handle = unsafe { codes_keys_iterator_new(self.message_handle, 0, "")? };
        let next_item_exists = unsafe { codes_keys_iterator_next(iterator_handle)? };

        Ok(KeysIterator {
            parent_message: PhantomData,
            iterator_handle,
            next_item_exists,
        })
    }
}

impl FallibleIterator for KeysIterator<'_> {
    type Item = String;
    type Error = CodesError;

    fn next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        if self.next_item_exists {
            let key_name;
            let next_item_exists;

            unsafe {
                key_name = codes_keys_iterator_get_name(self.iterator_handle)?;
                next_item_exists = codes_keys_iterator_next(self.iterator_handle)?;
            }

            self.next_item_exists = next_item_exists;

            Ok(Some(key_name))
        } else {
            Ok(None)
        }
    }
}

#[doc(hidden)]
impl Drop for KeysIterator<'_> {
    fn drop(&mut self) {
        unsafe {
            codes_keys_iterator_delete(self.iterator_handle).unwrap_or_else(|error| {
                warn!(
                    "codes_keys_iterator_delete() returned an error: {:?}",
                    &error
                );
            });
        }

        self.iterator_handle = null_mut();
    }
}

#[cfg(test)]
mod tests {
    use anyhow::{Context, Result};

    use crate::codes_handle::{CodesHandle, ProductKind};
    use crate::{FallibleIterator, FallibleStreamingIterator};
    use std::path::Path;

    use super::KeysIteratorFlags;

    #[test]
    fn keys_iterator_parameters() -> Result<()> {
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;
        let current_message = handle.next()?.context("Message not some")?;

        let flags = [
            KeysIteratorFlags::AllKeys,        //0
            KeysIteratorFlags::SkipOptional,   //2
            KeysIteratorFlags::SkipReadOnly,   //1
            KeysIteratorFlags::SkipDuplicates, //32
        ];
        let namespace = "geography";

        let mut kiter = current_message.new_keys_iterator(&flags, namespace)?;

        while let Some(key_name) = kiter.next()? {
            assert!(!key_name.is_empty());
        }

        Ok(())
    }

    #[test]
    fn invalid_namespace() -> Result<()> {
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;
        let current_message = handle.next()?.context("Message not some")?;

        let flags = vec![
            KeysIteratorFlags::AllKeys, //0
        ];

        let namespace = "blabla";

        let mut kiter = current_message.new_keys_iterator(&flags, namespace)?;

        while let Some(key_name) = kiter.next()? {
            assert!(!key_name.is_empty());
        }

        Ok(())
    }

    #[test]
    fn destructor() -> Result<()> {
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;
        let current_message = handle.next()?.context("Message not some")?;

        let _kiter = current_message.default_keys_iterator()?;

        drop(_kiter);

        testing_logger::validate(|captured_logs| {
            assert_eq!(captured_logs.len(), 1);
            assert_eq!(captured_logs[0].body, "codes_keys_iterator_delete");
            assert_eq!(captured_logs[0].level, log::Level::Trace);
        });

        drop(handle);

        testing_logger::validate(|captured_logs| {
            assert_eq!(captured_logs.len(), 1);
            assert_eq!(captured_logs[0].body, "codes_handle_delete");
            assert_eq!(captured_logs[0].level, log::Level::Trace);
        });

        Ok(())
    }
}
