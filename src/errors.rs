use std::time::SystemTimeError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error("could not deserialize toml file")]
    TOMLDeserializeError(#[from] toml::de::Error),

    #[error(transparent)]
    NotificationError(#[from]  notify_rust::error::Error),
    
    #[error(transparent)]
    DesynchronizedTimerError(#[from] SystemTimeError),

    #[error("could not parse session duration")]
    SessionDurationParserError,

    #[error(transparent)]
    RegexError(#[from] regex::Error),

    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
}