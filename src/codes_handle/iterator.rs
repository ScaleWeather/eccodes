use std::ptr;

use fallible_streaming_iterator::FallibleStreamingIterator;

use crate::{
    errors::CodesError,
    intermediate_bindings::{codes_handle_delete, codes_handle_new_from_file},
    CodesHandle, KeyedMessage,
};
#[cfg(feature = "experimental_index")]
use crate::{intermediate_bindings::codes_handle_new_from_index, CodesIndex};

use super::GribFile;

/// # Errors
///
/// The `advance()` and `next()` methods will return [`CodesInternal`](crate::errors::CodesInternal)
/// when internal ecCodes function returns non-zero code.
impl FallibleStreamingIterator for CodesHandle<GribFile> {
    type Item = KeyedMessage;

    type Error = CodesError;

    fn advance(&mut self) -> Result<(), Self::Error> {
        unsafe {
            codes_handle_delete(self.unsafe_message.message_handle)?;
        }

        // nullify message handle so that destructor is harmless
        // it might be excessive but it follows the correct pattern
        self.unsafe_message.message_handle = ptr::null_mut();

        let new_eccodes_handle =
            unsafe { codes_handle_new_from_file(self.source.pointer, self.product_kind)? };

        self.unsafe_message = KeyedMessage {
            message_handle: new_eccodes_handle,
        };

        Ok(())
    }

    fn get(&self) -> Option<&Self::Item> {
        if self.unsafe_message.message_handle.is_null() {
            None
        } else {
            Some(&self.unsafe_message)
        }
    }
}

#[cfg(feature = "experimental_index")]
#[cfg_attr(docsrs, doc(cfg(feature = "experimental_index")))]
/// # Errors
///
/// The `advance()` and `next()` methods will return [`CodesInternal`](crate::errors::CodesInternal)
/// when internal ecCodes function returns non-zero code.
impl FallibleStreamingIterator for CodesHandle<CodesIndex> {
    type Item = KeyedMessage;

    type Error = CodesError;

    fn advance(&mut self) -> Result<(), Self::Error> {
        unsafe {
            codes_handle_delete(self.unsafe_message.message_handle)?;
        }

        // nullify message handle so that destructor is harmless
        // it might be excessive but it follows the correct pattern
        self.unsafe_message.message_handle = ptr::null_mut();

        let new_eccodes_handle = unsafe { codes_handle_new_from_index(self.source.pointer)? };

        self.unsafe_message = KeyedMessage {
            message_handle: new_eccodes_handle,
        };

        Ok(())
    }

    fn get(&self) -> Option<&Self::Item> {
        if self.unsafe_message.message_handle.is_null() {
            None
        } else {
            Some(&self.unsafe_message)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        codes_handle::{CodesHandle, ProductKind},
        DynamicKeyType,
    };
    use anyhow::{Context, Ok, Result};
    use fallible_streaming_iterator::FallibleStreamingIterator;
    use std::path::Path;

    #[test]
    fn iterator_lifetimes() -> Result<()> {
        let file_path = Path::new("./data/iceland-levels.grib");
        let product_kind = ProductKind::GRIB;
        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;

        let msg1 = handle.next()?.context("Message not some")?;
        let key1 = msg1.read_key_dynamic("typeOfLevel")?;

        let msg2 = handle.next()?.context("Message not some")?;
        let key2 = msg2.read_key_dynamic("typeOfLevel")?;

        let msg3 = handle.next()?.context("Message not some")?;
        let key3 = msg3.read_key_dynamic("typeOfLevel")?;

        assert_eq!(key1, DynamicKeyType::Str("isobaricInhPa".to_string()));
        assert_eq!(key2, DynamicKeyType::Str("isobaricInhPa".to_string()));
        assert_eq!(key3, DynamicKeyType::Str("isobaricInhPa".to_string()));

        Ok(())
    }

    #[test]
    fn iterator_fn() -> Result<()> {
        let file_path = Path::new("./data/iceland-surface.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;

        while let Some(msg) = handle.next()? {
            let key = msg.read_key_dynamic("shortName")?;

            match key {
                DynamicKeyType::Str(_) => {}
                _ => panic!("Incorrect variant of string key"),
            }
        }

        Ok(())
    }

    #[test]
    fn iterator_collected() -> Result<()> {
        let file_path = Path::new("./data/iceland-surface.grib");
        let product_kind = ProductKind::GRIB;
        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;

        let mut handle_collected = vec![];

        while let Some(msg) = handle.next()? {
            handle_collected.push(msg.try_clone()?);
        }

        for msg in handle_collected {
            let key: DynamicKeyType = msg.read_key_dynamic("name")?;
            match key {
                DynamicKeyType::Str(_) => {}
                _ => panic!("Incorrect variant of string key"),
            }
        }

        Ok(())
    }

    #[test]
    fn iterator_return() -> Result<()> {
        let file_path = Path::new("./data/iceland-surface.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;
        let current_message = handle.next()?.context("Message not some")?;

        assert!(!current_message.message_handle.is_null());

        Ok(())
    }

    #[test]
    fn iterator_beyond_none() -> Result<()> {
        let file_path = Path::new("./data/iceland-surface.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;

        assert!(handle.next()?.is_some());
        assert!(handle.next()?.is_some());
        assert!(handle.next()?.is_some());
        assert!(handle.next()?.is_some());
        assert!(handle.next()?.is_some());

        assert!(handle.next()?.is_none());
        assert!(handle.next()?.is_none());
        assert!(handle.next()?.is_none());
        assert!(handle.next()?.is_none());

        Ok(())
    }

    #[test]
    fn iterator_filter() -> Result<()> {
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;

        // Use iterator to get a Keyed message with shortName "msl" and typeOfLevel "surface"
        // First, filter and collect the messages to get those that we want
        let mut level = vec![];

        while let Some(msg) = handle.next()? {
            if msg.read_key_dynamic("shortName")? == DynamicKeyType::Str("msl".to_string())
                && msg.read_key_dynamic("typeOfLevel")?
                    == DynamicKeyType::Str("surface".to_string())
            {
                level.push(msg.try_clone()?);
            }
        }

        // Now unwrap and access the first and only element of resulting vector
        // Find nearest modifies internal KeyedMessage fields so we need mutable reference
        let level = &level[0];

        println!("{:?}", level.read_key_dynamic("shortName"));

        // Get the four nearest gridpoints of Reykjavik
        let nearest_gridpoints = level.codes_nearest()?.find_nearest(64.13, -21.89)?;

        // Print value and distance of the nearest gridpoint
        println!(
            "value: {}, distance: {}",
            nearest_gridpoints[3].value, nearest_gridpoints[3].distance
        );

        Ok(())
    }
}
