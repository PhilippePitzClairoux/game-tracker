use std::ffi::OsString;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Errors {
    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error("could not deserialize toml file")]
    TOMLDeserializeError(#[from] toml::de::Error),
}