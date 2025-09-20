use fallible_iterator::FallibleIterator;

use crate::{CodesHandle, KeyedMessage, codes_handle::HandleGenerator, errors::CodesError};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct KeyedMessageGenerator<'a, S: HandleGenerator> {
    codes_handle: &'a mut CodesHandle<S>,
}

impl<S: HandleGenerator> CodesHandle<S> {
    pub fn message_generator<'a>(&'a mut self) -> KeyedMessageGenerator<'a, S> {
        KeyedMessageGenerator { codes_handle: self }
    }
}

/// # Errors
///
/// The `next()` will return [`CodesInternal`](crate::errors::CodesInternal)
/// when internal ecCodes function returns non-zero code.
impl<'ch, S: HandleGenerator> FallibleIterator for KeyedMessageGenerator<'ch, S> {
    type Item = KeyedMessage<'ch>;
    type Error = CodesError;

    fn next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        let new_eccodes_handle = self.codes_handle.source.gen_codes_handle()?;

        if new_eccodes_handle.is_null() {
            Ok(None)
        } else {
            Ok(Some(KeyedMessage {
                parent_message: PhantomData,
                message_handle: new_eccodes_handle,
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        DynamicKeyType, FallibleIterator,
        codes_handle::{CodesHandle, ProductKind},
    };
    use anyhow::{Context, Ok, Result};
    use std::path::Path;

    #[test]
    fn iterator_lifetimes() -> Result<()> {
        let file_path = Path::new("./data/iceland-levels.grib");
        let product_kind = ProductKind::GRIB;
        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;

        let msg1 = handle
            .message_generator()
            .next()?
            .context("Message not some")?;
        let key1 = msg1.read_key_dynamic("typeOfLevel")?;
        drop(msg1);

        let msg2 = handle
            .message_generator()
            .next()?
            .context("Message not some")?;
        let key2 = msg2.read_key_dynamic("typeOfLevel")?;
        drop(msg2);

        let msg3 = handle
            .message_generator()
            .next()?
            .context("Message not some")?;
        let key3 = msg3.read_key_dynamic("typeOfLevel")?;
        drop(msg3);

        assert_eq!(key1, DynamicKeyType::Str("isobaricInhPa".to_string()));
        assert_eq!(key2, DynamicKeyType::Str("isobaricInhPa".to_string()));
        assert_eq!(key3, DynamicKeyType::Str("isobaricInhPa".to_string()));

        Ok(())
    }

    #[test]
    fn message_lifetime_safety() -> Result<()> {
        let file_path = Path::new("./data/iceland-levels.grib");
        let product_kind = ProductKind::GRIB;
        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;
        let mut mgen = handle.message_generator();

        let msg1 = mgen.next()?.context("Message not some")?;
        drop(msg1);
        let msg2 = mgen.next()?.context("Message not some")?;
        let msg3 = mgen.next()?.context("Message not some")?;
        drop(msg3);
        let msg4 = mgen.next()?.context("Message not some")?;
        let msg5 = mgen.next()?.context("Message not some")?;
        drop(msg5);

        drop(mgen);
        // drop(handle); <- this is not allowed

        let key2 = msg2.read_key_dynamic("typeOfLevel")?;
        let key4 = msg4.read_key_dynamic("typeOfLevel")?;

        assert_eq!(key2, DynamicKeyType::Str("isobaricInhPa".to_string()));
        assert_eq!(key4, DynamicKeyType::Str("isobaricInhPa".to_string()));

        Ok(())
    }

    #[test]
    fn iterator_fn() -> Result<()> {
        let file_path = Path::new("./data/iceland-surface.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;

        while let Some(msg) = handle.message_generator().next()? {
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

        while let Some(msg) = handle.message_generator().next()? {
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
        let current_message = handle
            .message_generator()
            .next()?
            .context("Message not some")?;

        assert!(!current_message.message_handle.is_null());

        Ok(())
    }

    #[test]
    fn iterator_beyond_none() -> Result<()> {
        let file_path = Path::new("./data/iceland-surface.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesHandle::new_from_file(file_path, product_kind)?;
        let mut mgen = handle.message_generator();

        assert!(mgen.next()?.is_some());
        assert!(mgen.next()?.is_some());
        assert!(mgen.next()?.is_some());
        assert!(mgen.next()?.is_some());
        assert!(mgen.next()?.is_some());

        assert!(mgen.next()?.is_none());
        assert!(mgen.next()?.is_none());
        assert!(mgen.next()?.is_none());
        assert!(mgen.next()?.is_none());

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

        while let Some(msg) = handle.message_generator().next()? {
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
