use errno::Errno;
use thiserror::Error;

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

#[derive(Clone, Error, Debug)]
pub enum CodesInternal {
    #[error("Internal ecCodes error occured with code")]
    Test,
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
