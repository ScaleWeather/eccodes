use thiserror::Error;

#[derive(Copy, Clone, Error, Debug)]
pub enum CodesError {
    #[error("Internal ecCodes error occured")]
    Internal,
    #[error("Internal libc error occured")]
    Libc(#[from] LibcError),
}

#[derive(Copy, Clone, Error, Debug)]
pub enum LibcError {
    #[error("Internal libc error occured")]
    NullPtr,
}
