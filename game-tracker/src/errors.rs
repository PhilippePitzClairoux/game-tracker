use std::num::ParseFloatError;
use std::time::SystemTimeError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {

    #[error("could not parse session duration")]
    SessionDurationParserError,

    #[error("could not deserialize toml file")]
    TOMLDeserializeError(#[from] toml::de::Error),

    #[error("could not calculate when tomorrow is")]
    CalculateEndOfDayError,

    #[error("threshold value must be between 0 and 100")]
    InvalidThresholdError,

    #[error(transparent)]
    ParseFloatError(#[from] ParseFloatError),

    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error(transparent)]
    NotificationError(#[from]  notify_rust::error::Error),

    #[error(transparent)]
    RegexError(#[from] regex::Error),

    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error(transparent)]
    DatabaseError(#[from] rusqlite::Error),

    #[error(transparent)]
    TimedExecutionTamperingError(#[from] tampering_profiler_support::Errors),

    #[error(transparent)]
    TimeTamperingError(#[from] TimeTampering),
}

#[derive(Error, Debug)]
pub enum TimeTampering {
    #[error("clock tampering detected")]
    ClockTamperingError,

    #[error(transparent)]
    DesynchronizedTimerError(#[from] SystemTimeError),
}