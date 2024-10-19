use std::{
    io::{BufWriter, IntoInnerError},
    num::ParseIntError,
    sync::PoisonError,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    InvalidFilterName(String),
    #[error("{0}")]
    GrepChar(u8),
    #[error("{0}")]
    MinimumOutputLineLength(usize, usize),
    #[error("{0}")]
    ScannerMinimumOutputLineLength(String, usize, usize),
    #[error("{0}")]
    ScanerGrepCode(String, u8),
    #[error("{0}")]
    Encoding(String),
    #[error("{0}")]
    TooManyEncodings(String),
    #[error("{0}")]
    ParseInt(#[from] ParseIntError),
    #[error("{0}")]
    Poison(#[from] PoisonError<BufWriter<Vec<u8>>>),
    #[error("{0}")]
    IntoInner(IntoInnerError<BufWriter<Vec<u8>>>),
    #[error("unknown data store error")]
    Unknown,
}
