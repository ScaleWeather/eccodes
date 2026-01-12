//! Definition of `CodesMessage` and its associated functions
//! used for reading and writing data of given variable from GRIB file

mod clone;
#[cfg(feature = "ndarray")]
#[cfg_attr(docsrs, doc(cfg(feature = "ndarray")))]
mod ndarray;
mod read;
mod write;

#[cfg_attr(docsrs, doc(cfg(feature = "ndarray")))]
pub use ndarray::RustyCodesMessage;
pub use read::{DynamicKeyType, KeyPropertiesRead, KeyRead};
pub use write::KeyWrite;

use eccodes_sys::codes_handle;
use std::{
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
    ptr::null_mut,
    sync::{Arc, Mutex},
};
use tracing::{Level, event, instrument};

use crate::{CodesFile, intermediate_bindings::codes_handle_delete};

/// Base structure for [`RefMessage`], [`ArcMessage`] and [`BufMessage`] that provides access to the data
/// contained in the GRIB file, which directly corresponds to the message in the GRIB file.
///
/// `RefMessage` comes with no performance overhead but has its liftime tied to its parent `CodesFile`.
///
/// `ArcMessage` can be moved and shared across threads.
///
/// `BufMessage` is independent of its parent and can be edited with [`KeyWrite`] methods.  
///
/// **Usage examples are provided in documentation of implemented methods.**
///
/// You can think about the message as a container of data corresponding to a single variable
/// at given date, time and level. In ecCodes the message is represented as a collection of unique
/// key-value pairs.
///
/// You can read a `Key` with static types using [`read_key()`](KeyRead::read_key()) or with [`DynamicKeyType`] using[`read_key_dynamic()`](CodesMessage::read_key_dynamic())
/// To iterate over all key names use [`KeysIterator`](crate::KeysIterator). You can also modify the message using
/// [`write_key_unchecked()`](KeyWrite::write_key_unchecked()) (only available for `BufMessage`).
///
/// [`CodesNearest`](crate::CodesNearest) can be used to find nearest gridpoints for given coordinates in the `CodesMessage`.
///
/// To write the message to file use [`write_to_file()`](CodesMessage::write_to_file).
///
/// If you are interested only in getting data values from the message you can use
/// [`to_ndarray()`](CodesMessage::to_ndarray) available in `ndarray` feature.
///
/// Some of the useful keys are: `validityDate`, `validityTime`, `level`, `typeOfLevel`, `shortName`, `units` and `values`.
///
/// Note that names, types and availability of some keys can vary between platforms and ecCodes versions. You should test
/// your code whenever changing the environment.
///
/// Destructor for this structure does not panic, but some internal functions may rarely fail
/// leading to bugs. Errors encountered in desctructor the are reported via [`tracing`].
#[derive(Debug)]
pub struct CodesMessage<P: Debug> {
    pub(crate) _parent: P,
    pub(crate) message_handle: *mut codes_handle,
}

// This is a little unintuitive, but we use `()` here to not unnecessarily pollute
// CodesMessage and derived types with generics, because `PhantomData` is needed
// only for lifetime restriction and we tightly control how `CodesMessage` is created.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd)]
#[doc(hidden)]
pub struct RefParent<'ch>(PhantomData<&'ch ()>);

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd)]
#[doc(hidden)]
pub struct BufParent();

#[derive(Debug)]
#[doc(hidden)]
pub struct ArcParent<D: Debug> {
    _arc_handle: Arc<Mutex<CodesFile<D>>>,
}

/// [`CodesMessage`] that has its liftime tied to its parent `CodesFile`.
pub type RefMessage<'ch> = CodesMessage<RefParent<'ch>>;

/// [`CodesMessage`] that can be moved and shared across threads.
pub type ArcMessage<D> = CodesMessage<ArcParent<D>>;

unsafe impl<D: Debug> Send for ArcMessage<D> {}
unsafe impl<D: Debug> Sync for ArcMessage<D> {}

/// [`CodesMessage`] that is independent of its parent and can be edited with [`KeyWrite`] methods.
pub type BufMessage = CodesMessage<BufParent>;

unsafe impl Send for BufMessage {}
unsafe impl Sync for BufMessage {}

impl RefMessage<'_> {
    pub(crate) const fn new(handle: *mut codes_handle) -> Self {
        RefMessage {
            _parent: RefParent(PhantomData),
            message_handle: handle,
        }
    }
}

impl<D: Debug> ArcMessage<D> {
    pub(crate) fn new(handle: *mut codes_handle, parent: &Arc<Mutex<CodesFile<D>>>) -> Self {
        ArcMessage {
            _parent: ArcParent {
                _arc_handle: parent.clone(),
            },
            message_handle: handle,
        }
    }
}

impl BufMessage {
    pub(crate) const fn new(handle: *mut codes_handle) -> Self {
        BufMessage {
            _parent: BufParent(),
            message_handle: handle,
        }
    }
}

impl<P: Debug> Drop for CodesMessage<P> {
    /// Executes the destructor for this type.
    /// This method calls destructor functions from ecCodes library.
    /// In some edge cases these functions can return non-zero code.
    /// In such case all pointers and file descriptors are safely deleted.
    /// However memory leaks can still occur.
    ///
    /// If any function called in the destructor returns an error warning will appear in log/tracing.
    /// If bugs occur during `CodesMessage` drop please enable log output and post issue on [Github](https://github.com/ScaleWeather/eccodes).
    ///
    /// Technical note: delete functions in ecCodes can only fail with [`CodesInternalError`](crate::errors::CodesInternal::CodesInternalError)
    /// when other functions corrupt the inner memory of pointer, in that case memory leak is possible.
    /// In case of corrupt pointer segmentation fault will occur.
    /// The pointers are cleared at the end of drop as they are not functional regardless of result of delete functions.
    ///
    /// # Panics
    ///
    /// In debug mode when error is encountered.
    #[instrument(level = "trace")]
    fn drop(&mut self) {
        unsafe {
            codes_handle_delete(self.message_handle).unwrap_or_else(|error| {
                event!(
                    Level::ERROR,
                    "codes_handle_delete() returned an error: {:?}",
                    &error
                );
                debug_assert!(false, "Error in CodesMessage::drop");
            });
        }

        self.message_handle = null_mut();
    }
}

#[cfg(test)]
mod tests {
    use crate::codes_file::{CodesFile, ProductKind};
    use anyhow::{Context, Result};
    use fallible_iterator::FallibleIterator;
    use std::path::Path;

    #[test]
    fn check_docs_keys() -> Result<()> {
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesFile::new_from_file(file_path, product_kind)?;
        let current_message = handle
            .ref_message_iter()
            .next()?
            .context("Message not some")?;

        let _ = current_message.read_key_dynamic("validityDate")?;
        let _ = current_message.read_key_dynamic("validityTime")?;
        let _ = current_message.read_key_dynamic("level")?;
        let _ = current_message.read_key_dynamic("shortName")?;
        let _ = current_message.read_key_dynamic("units")?;
        let _ = current_message.read_key_dynamic("values")?;
        let _ = current_message.read_key_dynamic("typeOfLevel")?;

        Ok(())
    }

    #[test]
    fn message_clone_1() -> Result<()> {
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesFile::new_from_file(file_path, product_kind)?;
        let current_message = handle
            .ref_message_iter()
            .next()?
            .context("Message not some")?;
        let cloned_message = current_message.try_clone()?;

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

        let mut handle = CodesFile::new_from_file(file_path, product_kind)?;
        let mut mgen = handle.ref_message_iter();
        let msg = mgen.next()?.context("Message not some")?.try_clone()?;
        let _ = mgen.next()?;

        drop(handle);

        let _ = msg.read_key_dynamic("dataDate")?;
        let _ = msg.read_key_dynamic("jDirectionIncrementInDegrees")?;
        let _ = msg.read_key_dynamic("values")?;
        let _ = msg.read_key_dynamic("name")?;
        let _ = msg.read_key_dynamic("section1Padding")?;
        let _ = msg.read_key_dynamic("experimentVersionNumber")?;

        Ok(())
    }

    #[test]
    fn message_clone_drop() -> Result<()> {
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesFile::new_from_file(file_path, product_kind)?;
        let _msg_ref = handle
            .ref_message_iter()
            .next()?
            .context("Message not some")?;
        let _msg_clone = _msg_ref.try_clone()?;

        drop(_msg_ref);
        drop(handle);
        drop(_msg_clone);

        Ok(())
    }

    #[test]
    fn ref_message_drop_null() -> Result<()> {
        let file_path = Path::new("./data/iceland.grib");
        let product_kind = ProductKind::GRIB;

        let mut handle = CodesFile::new_from_file(file_path, product_kind)?;
        let mut current_message = handle
            .ref_message_iter()
            .next()?
            .context("Message not some")?;
        current_message.message_handle = std::ptr::null_mut();

        Ok(())
    }
}
