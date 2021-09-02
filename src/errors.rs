use errno::Errno;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CodesError {
    #[error("Internal ecCodes error occured with code {0}")]
    Internal(i32),
    #[error("Internal libc error occured")]
    Libc(#[from] LibcError),
    #[error("Provided file has no extension")]
    NoExtension,
    #[error("Provided file has incorrect extension")]
    WrongExtension,
    #[error("Error occured while opening the file")]
    CantOpenFile(#[from] std::io::Error)
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
