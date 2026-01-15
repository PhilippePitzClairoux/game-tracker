use std::time::SystemTimeError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Errors {
    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error("could not deserialize toml file")]
    TOMLDeserializeError(#[from] toml::de::Error),

    #[error(transparent)]
    NotificationError(#[from]  notify_rust::error::Error),
    
    #[error(transparent)]
    DesynchronizedTimerError(#[from] SystemTimeError)
}