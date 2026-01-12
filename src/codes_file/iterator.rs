use fallible_iterator::FallibleIterator;

use crate::{ArcMessage, CodesFile, RefMessage, errors::CodesError};
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

/// Iterator over messages in `CodesFile` which returns [`RefMessage`] with lifetime tied to the `CodesFile`.
///
/// This structure implements [`FallibleIterator`] which allows you to iterate over messages in the file.
/// The iterator returns [`RefMessage`] with lifetime tied to the lifetime of `CodesFile`, that is `RefMessage`
/// cannot outlive the `CodesFile` it was generated from.
///
/// Creating this iter requires `CodesFile` to be mutable.
///
/// If you need to share the message(s) across threads use [`ArcMessageIter`].
///
/// If you need a longer lifetime or want to modify the message, use [`try_clone()`](RefMessage::try_clone).
///
/// ## Example
///
/// ```
/// use eccodes::{CodesFile, ProductKind, FallibleIterator};
/// #
/// # fn main() -> anyhow::Result<()> {
///     // Open the file
///     let mut file = CodesFile::new_from_file("./data/iceland.grib", ProductKind::GRIB)?;
///
///     // Create RefMessageIter
///     let mut msg_iter = file.ref_message_iter();
///
///     // Now we can access the first message with next()
///     // Note that FallibleIterator must be in scope
///     let _msg = msg_iter.next()?;
///
///     // Note we cannot drop the file here, because we first need to drop the message
///     // drop(file);
/// #     Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct RefMessageIter<'a, D: Debug> {
    codes_file: &'a mut CodesFile<D>,
}

impl<D: Debug> CodesFile<D> {
    /// Generates [`RefMessageIter`] that allows to access messages as references to their parent file.
    pub fn ref_message_iter(&mut self) -> RefMessageIter<'_, D> {
        RefMessageIter { codes_file: self }
    }
}

impl<'ch, D: Debug> FallibleIterator for RefMessageIter<'ch, D> {
    type Item = RefMessage<'ch>;
    type Error = CodesError;

    /// # Errors
    ///
    /// The method will return [`CodesInternal`](crate::errors::CodesInternal)
    /// when internal ecCodes function returns non-zero code.
    fn next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        let eccodes_handle = self.codes_file.generate_codes_handle()?;

        if eccodes_handle.is_null() {
            Ok(None)
        } else {
            Ok(Some(RefMessage::new(eccodes_handle)))
        }
    }
}

/// Iterator over messages in `CodesFile` which returns [`ArcMessage`] which can be shared across threads.
///
/// `ArcMessage` implements `Send + Sync` so it can be both moved to thread (for example, to read messages in parallel)
/// or shared across threads (when wrapped in [`Arc`]).
/// 
/// This structure implements [`FallibleIterator`] - see the documentation for information how that differs from a standard `Iter``.
///
/// Creating this iter does not require `CodesFile` to be mutable, because it takes ownership over the `CodesFile`.
///
/// If you don't need to share the message, use [`RefMessageIter`] to avoid the performance overhead of [`Arc`].
/// 
/// If you want to modify the message, use [`try_clone()`](RefMessage::try_clone).
/// 
/// ## Example
/// 
/// See the second example in the main crate description for example usage of `ArcMessageIter`.
#[derive(Debug)]
pub struct ArcMessageIter<D: Debug> {
    codes_file: Arc<Mutex<CodesFile<D>>>,
}
impl<D: Debug> CodesFile<D> {
    /// Generates [`ArcMessageIter`] that allows to access messages that can be shared across threads.
    pub fn arc_message_iter(self) -> ArcMessageIter<D> {
        ArcMessageIter {
            codes_file: Arc::new(Mutex::new(self)),
        }
    }
}

impl<D: Debug> FallibleIterator for ArcMessageIter<D> {
    type Item = ArcMessage<D>;

    type Error = CodesError;

    /// # Errors
    ///
    /// The method will return [`CodesInternal`](crate::errors::CodesInternal)
    /// when internal ecCodes function returns non-zero code.
    /// 
    /// # Panics
    /// 
    /// This method internally uses a Mutex to access `CodesFile`, which can panic when poisoned,
    /// but thers is no path in which you can get to the state of poisoned mutex, while still able to access this method.
    fn next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        let eccodes_handle = self
            .codes_file
            .lock()
            // This mutex can be poisoned only when thread that holds ArcMessageIter panics, which would make using the mutex impossible")
            .expect("The mutex inside ArcMessageIter got poisoned")
            .generate_codes_handle()?;

        if eccodes_handle.is_null() {
            Ok(None)
        } else {
            Ok(Some(ArcMessage::new(eccodes_handle, &self.codes_file)))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        FallibleIterator,
        codes_file::{CodesFile, ProductKind},
        codes_message::DynamicKeyType,
    };
    use anyhow::{Context, Ok, Result};
    use float_cmp::assert_approx_eq;
    use std::{
        path::Path,
        sync::{Arc, Barrier},
    };

    #[test]
    fn iterator_lifetimes() -> Result<()> {
        let file_path = Path::new("./data/iceland-levels.grib");
        let product_kind = ProductKind::GRIB;
        let mut handle = CodesFile::new_from_file(file_path, product_kind)?;

        let msg1 = handle
            .ref_message_iter()
            .next()?
            .context("Message not some")?;
        let key1 = msg1.read_key_dynamic("typeOfLevel")?;
        drop(msg1);

        let msg2 = handle
            .ref_message_iter()
            .next()?
            .context("Message not some")?;
        let key2 = msg2.read_key_dynamic("typeOfLevel")?;
        drop(msg2);

        let msg3 = handle
            .ref_message_iter()
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
        let mut handle = CodesFile::new_from_file(file_path, product_kind)?;
        let msg2;
        let msg4;

        {
            let mut mgen = handle.ref_message_iter();

            let msg1 = mgen.next()?.context("Message not some")?;
            drop(msg1);
            msg2 = mgen.next()?.context("Message not some")?;
            let msg3 = mgen.next()?.context("Message not some")?;
            drop(msg3);
            msg4 = mgen.next()?.context("Message not some")?;
            let msg5 = mgen.next()?.context("Message not some")?;
            drop(msg5);
        }
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

        let mut handle = CodesFile::new_from_file(file_path, product_kind)?;

        while let Some(msg) = handle.ref_message_iter().next()? {
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
        let mut handle = CodesFile::new_from_file(file_path, product_kind)?;

        let mut handle_collected = vec![];

        while let Some(msg) = handle.ref_message_iter().next()? {
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

        let mut handle = CodesFile::new_from_file(file_path, product_kind)?;
        let current_message = handle
            .ref_message_iter()
            .next()?
            .context("Message not some")?;

        assert!(!current_message.message_handle.is_null());

        Ok(())
    }

    #[test]
    fn iterator_beyond_none() -> Result<()> {
        let file_path = Path::new("./data/iceland-surface.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesFile::new_from_file(file_path, product_kind)?;
        let mut mgen = handle.ref_message_iter();

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

        let mut handle = CodesFile::new_from_file(file_path, product_kind)?;

        // Use iterator to get a Keyed message with shortName "msl" and typeOfLevel "surface"
        // First, filter and collect the messages to get those that we want
        let mut level = vec![];

        while let Some(msg) = handle.ref_message_iter().next()? {
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

        let short_name = level.read_key_dynamic("shortName")?;
        assert_eq!(short_name, DynamicKeyType::Str("msl".into()));

        // Get the four nearest gridpoints of Reykjavik
        let nearest_gridpoints = level.codes_nearest()?.find_nearest(64.13, -21.89)?;
        let value = nearest_gridpoints[3].value;
        let distance = nearest_gridpoints[3].distance;

        assert_approx_eq!(f64, value, 100557.9375);
        assert_approx_eq!(f64, distance, 14.358879960775498);

        Ok(())
    }

    #[test]
    fn thread_safety_messsage_wise() -> Result<()> {
        let file_path = Path::new("./data/iceland-levels.grib");
        let product_kind = ProductKind::GRIB;

        let handle = CodesFile::new_from_file(file_path, product_kind)?;
        let mut mgen = handle.arc_message_iter();
        // let _ = handle.atomic_message_generator(); <- not allowed due to ownership

        let barrier = Arc::new(Barrier::new(10));

        let mut v = vec![];

        for _ in 0..10 {
            let msg = mgen.next()?.context("No more messages")?;
            let b = barrier.clone();

            let t = std::thread::spawn(move || {
                for _ in 0..10 {
                    b.wait();
                    for _ in 0..100 {
                        let _ = msg.read_key_dynamic("shortName").unwrap();
                    }
                }
            });

            v.push(t);
        }

        for th in v {
            th.join().unwrap();
        }

        Ok(())
    }

    #[test]
    fn thread_safety_within_message() -> Result<()> {
        let file_path = Path::new("./data/iceland-levels.grib");
        let product_kind = ProductKind::GRIB;

        let handle = CodesFile::new_from_file(file_path, product_kind)?;
        let mut mgen = handle.arc_message_iter();
        let msg = Arc::new(mgen.next()?.context("No more messages")?);

        let barrier = Arc::new(Barrier::new(10));

        let mut v = vec![];

        for _ in 0..10 {
            let msg_inner = msg.clone();
            let b = barrier.clone();

            let t = std::thread::spawn(move || {
                for _ in 0..10 {
                    b.wait();
                    for _ in 0..100 {
                        let _ = msg_inner.read_key_dynamic("shortName").unwrap();
                    }
                }
            });

            v.push(t);
        }

        for th in v {
            th.join().unwrap();
        }

        Ok(())
    }
}
