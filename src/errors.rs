//!Module containing all error types used by the crate
//!
//!This crate uses [`thiserror`] crate to define its error types.
//!
//!If you encounter an error that you believe is a result of implementation bug
//!rather then user's mistake post an issue on Github.

use errno::Errno;
use num_derive::FromPrimitive;
use thiserror::Error;

#[derive(Error, Debug)]
///Errors returned by the crate's functions.
///These are the only errors that the user may face.
pub enum CodesError {
    ///Returned when ecCodes library function returns an error code.
    ///Check [`CodesInternal`] for more details.
    #[error("ecCodes function returned a non-zero code")]
    Internal(#[from] CodesInternal),

    ///Returned when one of libc functions returns a non-zero error code.
    ///Check libc documentation for details of the errors.
    ///For libc reference check these websites: ([1](https://man7.org/linux/man-pages/index.html))
    ///([2](https://pubs.opengroup.org/onlinepubs/9699919799/functions/contents.html))
    #[error("libc function returned an error with errno {0} and code {0}")]
    LibcNonZero(i32, Errno),

    ///Returned when there is an issue while opening the file.
    ///Check the [`std::fs`] documentation why and when this error can occur.
    #[error("Error occured while opening the file")]
    CantOpenFile(#[from] std::io::Error),

    ///Returned when the constructor did not find any message of requested kind
    #[error("No message have been found in the file")]
    NoMessages,

    ///Returned when the string cannot be parsed as valid UTF8 string.
    #[error(transparent)]
    CstrUTF8(#[from] std::str::Utf8Error),

    ///Returned when the C-string returned by ecCodes library cannot be converted
    ///into a Rust-string.
    #[error(transparent)]
    NulChar(#[from] std::ffi::FromBytesWithNulError),
}

#[derive(Copy, Eq, PartialEq, Clone, Ord, PartialOrd, Hash, Error, Debug, FromPrimitive)]
///Errors returned by internal ecCodes library functions.
///Copied directly from the ecCodes API.
pub enum CodesInternal {
    ///No error
    #[error("No error")]
    CodesSuccess = 0,

    ///End of resource reached
    #[error("End of resource reached")]
    CodesEndOfFile = -1,

    ///Internal error
    #[error("Internal error")]
    CodesInternalError = -2,

    ///Passed buffer is too small
    #[error("Passed buffer is too small")]
    CodesBufferTooSmall = -3,

    ///Function not yet implemented
    #[error("Function not yet implemented")]
    CodesNotImplemented = -4,

    ///Missing 7777 at end of message
    #[error("Missing 7777 at end of message")]
    Codes7777NotFound = -5,

    ///Passed array is too small
    #[error("Passed array is too small")]
    CodesArrayTooSmall = -6,

    ///File not found
    #[error("File not found")]
    CodesFileNotFound = -7,

    ///Code not found in code table
    #[error("Code not found in code table")]
    CodesCodeNotFoundInTable = -8,

    ///Array size mismatch
    #[error("Array size mismatch")]
    CodesWrongArraySize = -9,

    ///Key/value not found
    #[error("Key/value not found")]
    CodesNotFound = -10,

    ///Input output problem
    #[error("Input output problem")]
    CodesIoProblem = -11,

    ///Message invalid
    #[error("Message invalid")]
    CodesInvalidMessage = -12,

    ///Decoding invalid
    #[error("Decoding invalid")]
    CodesDecodingError = -13,

    ///Encoding invalid
    #[error("Encoding invalid")]
    CodesEncodingError = -14,

    ///Code cannot unpack because of string too small
    #[error("Code cannot unpack because of string too small")]
    CodesNoMoreInSet = -15,

    ///Problem with calculation of geographic attributes
    #[error("Problem with calculation of geographic attributes")]
    CodesGeocalculusProblem = -16,

    ///Memory allocation error
    #[error("Memory allocation error")]
    CodesOutOfMemory = -17,

    ///Value is read only
    #[error("Value is read only")]
    CodesReadOnly = -18,

    ///Invalid argument
    #[error("Invalid argument")]
    CodesInvalidArgument = -19,

    ///Null handle
    #[error("Null handle")]
    CodesNullHandle = -20,

    ///Invalid section number
    #[error("Invalid section number")]
    CodesInvalidSectionNumber = -21,

    ///Value cannot be missing
    #[error("Value cannot be missing")]
    CodesValueCannotBeMissing = -22,

    ///Wrong message length
    #[error("Wrong message length")]
    CodesWrongLength = -23,

    ///Invalid key type
    #[error("Invalid key type")]
    CodesInvalidType = -24,

    ///Unable to set step
    #[error("Unable to set step")]
    CodesWrongStep = -25,

    ///Wrong units for step (step must be integer)
    #[error("Wrong units for step (step must be integer)")]
    CodesWrongStepUnit = -26,

    ///Invalid file id
    #[error("Invalid file id")]
    CodesInvalidFile = -27,

    ///Invalid grib id
    #[error("Invalid grib id")]
    CodesInvalidGrib = -28,

    ///Invalid index id
    #[error("Invalid index id")]
    CodesInvalidIndex = -29,

    ///Invalid iterator id
    #[error("Invalid iterator id")]
    CodesInvalidIterator = -30,

    ///Invalid keys iterator id
    #[error("Invalid keys iterator id")]
    CodesInvalidKeysIterator = -31,

    ///Invalid nearest id
    #[error("Invalid nearest id")]
    CodesInvalidNearest = -32,

    ///Invalid order by
    #[error("Invalid order by")]
    CodesInvalidOrderby = -33,

    ///Missing a key from the fieldset
    #[error("Missing a key from the fieldset")]
    CodesMissingKey = -34,

    ///The point is out of the grid area
    #[error("The point is out of the grid area")]
    CodesOutOfArea = -35,

    ///Concept no match
    #[error("Concept no match")]
    CodesConceptNoMatch = -36,

    ///Hash array no match
    #[error("Hash array no match")]
    CodesHashArrayNoMatch = -37,

    ///Definitions files not found
    #[error("Definitions files not found")]
    CodesNoDefinitions = -38,

    ///Wrong type while packing
    #[error("Wrong type while packing")]
    CodesWrongType = -39,

    ///End of resource
    #[error("End of resource")]
    CodesEnd = -40,

    ///Unable to code a field without values
    #[error("Unable to code a field without values")]
    CodesNoValues = -41,

    ///Grid description is wrong or inconsistent
    #[error("Grid description is wrong or inconsistent")]
    CodesWrongGrid = -42,

    ///End of index reached
    #[error("End of index reached")]
    CodesEndOfIndex = -43,

    ///Null index
    #[error("Null index")]
    CodesNullIndex = -44,

    ///End of resource reached when reading message
    #[error("End of resource reached when reading message")]
    CodesPrematureEndOfFile = -45,

    ///An internal array is too small
    #[error("An internal array is too small")]
    CodesInternalArrayTooSmall = -46,

    ///Message is too large for the current architecture
    #[error("Message is too large for the current architecture")]
    CodesMessageTooLarge = -47,

    ///Constant field
    #[error("Constant field")]
    CodesConstantField = -48,

    ///Switch unable to find a matching case
    #[error("Switch unable to find a matching case")]
    CodesSwitchNoMatch = -49,

    ///Underflow
    #[error("Underflow")]
    CodesUnderflow = -50,

    ///Message malformed
    #[error("Message malformed")]
    CodesMessageMalformed = -51,

    ///Index is corrupted
    #[error("Index is corrupted")]
    CodesCorruptedIndex = -52,

    ///Invalid number of bits per value
    #[error("Invalid number of bits per value")]
    CodesInvalidBpv = -53,

    ///Edition of two messages is different
    #[error("Edition of two messages is different")]
    CodesDifferentEdition = -54,

    ///Value is different
    #[error("Value is different")]
    CodesValueDifferent = -55,

    ///Invalid key value
    #[error("Invalid key value")]
    CodesInvalidKeyValue = -56,

    ///String is smaller than requested
    #[error("String is smaller than requested")]
    CodesStringTooSmall = -57,

    ///Wrong type conversion
    #[error("Wrong type conversion")]
    CodesWrongConversion = -58,

    ///Missing BUFR table entry for descriptor
    #[error("Missing BUFR table entry for descriptor")]
    CodesMissingBufrEntry = -59,

    ///Null pointer
    #[error("Null pointer")]
    CodesNullPointer = -60,

    ///Attribute is already present =  cannot add
    #[error("Attribute is already present =  cannot add")]
    CodesAttributeClash = -61,

    ///Too many attributes. Increase MAX_ACCESSOR_ATTRIBUTES
    #[error("Too many attributes. Increase MAX_ACCESSOR_ATTRIBUTES")]
    CodesTooManyAttributes = -62,

    ///Attribute not found
    #[error("Attribute not found")]
    CodesAttributeNotFound = -63,

    ///Edition not supported
    #[error("Edition not supported")]
    CodesUnsupportedEdition = -64,

    ///Value out of coding range
    #[error("Value out of coding range")]
    CodesOutOfRange = -65,

    ///Size of bitmap is incorrect
    #[error("Size of bitmap is incorrect")]
    CodesWrongBitmapSize = -66,

    ///Functionality not enabled
    #[error("Functionality not enabled")]
    CodesFunctionalityNotEnabled = -67,
}
