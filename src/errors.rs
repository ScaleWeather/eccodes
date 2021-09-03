use errno::Errno;
use thiserror::Error;
use num_derive::FromPrimitive;

#[derive(Error, Debug)]
pub enum CodesError {
    #[error("Internal ecCodes error occured with code")]
    Internal(#[from] CodesInternal),
    #[error("Internal libc error occured")]
    Libc(#[from] LibcError),
    #[error("Provided file has no extension")]
    NoFileExtension,
    #[error("Provided file has incorrect extension")]
    WrongFileExtension,
    #[error("Error occured while opening the file")]
    CantOpenFile(#[from] std::io::Error),
}

#[derive(Clone, Error, Debug, FromPrimitive)]
pub enum CodesInternal {
    #[error("No error")]
    CodesSuccess = 0,
    #[error("End of resource reached")]
    CodesEndOfFile = -1,
    #[error("Internal error")]
    CodesInternalError = -2,
    #[error("Passed buffer is too small")]
    CodesBufferTooSmall = -3,
    #[error("Function not yet implemented")]
    CodesNotImplemented = -4,
    #[error("Missing 7777 at end of message")]
    Codes7777NotFound = -5,
    #[error("Passed array is too small")]
    CodesArrayTooSmall = -6,
    #[error("File not found")]
    CodesFileNotFound = -7,
    #[error("Code not found in code table")]
    CodesCodeNotFoundInTable = -8,
    #[error("Array size mismatch")]
    CodesWrongArraySize = -9,
    #[error("Key/value not found")]
    CodesNotFound = -10,
    #[error("Input output problem")]
    CodesIoProblem = -11,
    #[error("Message invalid")]
    CodesInvalidMessage = -12,
    #[error("Decoding invalid")]
    CodesDecodingError = -13,
    #[error("Encoding invalid")]
    CodesEncodingError = -14,
    #[error("Code cannot unpack because of string too small")]
    CodesNoMoreInSet = -15,
    #[error("Problem with calculation of geographic attributes")]
    CodesGeocalculusProblem = -16,
    #[error("Memory allocation error")]
    CodesOutOfMemory = -17,
    #[error("Value is read only")]
    CodesReadOnly = -18,
    #[error("Invalid argument")]
    CodesInvalidArgument = -19,
    #[error("Null handle")]
    CodesNullHandle = -20,
    #[error("Invalid section number")]
    CodesInvalidSectionNumber = -21,
    #[error("Value cannot be missing")]
    CodesValueCannotBeMissing = -22,
    #[error("Wrong message length")]
    CodesWrongLength = -23,
    #[error("Invalid key type")]
    CodesInvalidType = -24,
    #[error("Unable to set step")]
    CodesWrongStep = -25,
    #[error("Wrong units for step (step must be integer)")]
    CodesWrongStepUnit = -26,
    #[error("Invalid file id")]
    CodesInvalidFile = -27,
    #[error("Invalid grib id")]
    CodesInvalidGrib = -28,
    #[error("Invalid index id")]
    CodesInvalidIndex = -29,
    #[error("Invalid iterator id")]
    CodesInvalidIterator = -30,
    #[error("Invalid keys iterator id")]
    CodesInvalidKeysIterator = -31,
    #[error("Invalid nearest id")]
    CodesInvalidNearest = -32,
    #[error("Invalid order by")]
    CodesInvalidOrderby = -33,
    #[error("Missing a key from the fieldset")]
    CodesMissingKey = -34,
    #[error("The point is out of the grid area")]
    CodesOutOfArea = -35,
    #[error("Concept no match")]
    CodesConceptNoMatch = -36,
    #[error("Hash array no match")]
    CodesHashArrayNoMatch = -37,
    #[error("Definitions files not found")]
    CodesNoDefinitions = -38,
    #[error("Wrong type while packing")]
    CodesWrongType = -39,
    #[error("End of resource")]
    CodesEnd = -40,
    #[error("Unable to code a field without values")]
    CodesNoValues = -41,
    #[error("Grid description is wrong or inconsistent")]
    CodesWrongGrid = -42,
    #[error("End of index reached")]
    CodesEndOfIndex = -43,
    #[error("Null index")]
    CodesNullIndex = -44,
    #[error("End of resource reached when reading message")]
    CodesPrematureEndOfFile = -45,
    #[error("An internal array is too small")]
    CodesInternalArrayTooSmall = -46,
    #[error("Message is too large for the current architecture")]
    CodesMessageTooLarge = -47,
    #[error("Constant field")]
    CodesConstantField = -48,
    #[error("Switch unable to find a matching case")]
    CodesSwitchNoMatch = -49,
    #[error("Underflow")]
    CodesUnderflow = -50,
    #[error("Message malformed")]
    CodesMessageMalformed = -51,
    #[error("Index is corrupted")]
    CodesCorruptedIndex = -52,
    #[error("Invalid number of bits per value")]
    CodesInvalidBpv = -53,
    #[error("Edition of two messages is different")]
    CodesDifferentEdition = -54,
    #[error("Value is different")]
    CodesValueDifferent = -55,
    #[error("Invalid key value")]
    CodesInvalidKeyValue = -56,
    #[error("String is smaller than requested")]
    CodesStringTooSmall = -57,
    #[error("Wrong type conversion")]
    CodesWrongConversion = -58,
    #[error("Missing BUFR table entry for descriptor")]
    CodesMissingBufrEntry = -59,
    #[error("Null pointer")]
    CodesNullPointer = -60,
    #[error("Attribute is already present =  cannot add")]
    CodesAttributeClash = -61,
    #[error("Too many attributes. Increase MAX_ACCESSOR_ATTRIBUTES")]
    CodesTooManyAttributes = -62,
    #[error("Attribute not found")]
    CodesAttributeNotFound = -63,
    #[error("Edition not supported")]
    CodesUnsupportedEdition = -64,
    #[error("Value out of coding range")]
    CodesOutOfRange = -65,
    #[error("Size of bitmap is incorrect")]
    CodesWrongBitmapSize = -66,
    #[error("Functionality not enabled")]
    CodesFunctionalityNotEnabled = -67,
}

#[derive(Clone, Error, Debug)]
pub enum LibcError {
    #[error("Libc function returned null pointer, errno code {0} with error {0}")]
    NullPtr(i32, Errno),

    #[error("Libc function returned non-zero code")]
    NonZero,

    #[error(transparent)]
    CStringNull(#[from] std::ffi::NulError),
}
