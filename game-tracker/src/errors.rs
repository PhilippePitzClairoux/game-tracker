use std::time::SystemTimeError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {

    #[error("could not parse session duration")]
    SessionDurationParserError,

    #[error("could not deserialize toml file")]
    TOMLDeserializeError(#[from] toml::de::Error),

    #[error("clock tampering detected")]
    ClockTamperingError,

    #[error("could not calculate when tomorrow is")]
    CalculateEndOfDayError,

    #[error("the end of the day (since start_time was set) has been reached")]
    EndOfDayReachedError,

    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error(transparent)]
    NotificationError(#[from]  notify_rust::error::Error),

    #[error(transparent)]
    DesynchronizedTimerError(#[from] SystemTimeError),

    #[error(transparent)]
    RegexError(#[from] regex::Error),

    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error(transparent)]
    ProfilerError(#[from] tampering_profiler_support::Errors),

    #[error(transparent)]
    DatabaseError(#[from] rusqlite::Error),
}