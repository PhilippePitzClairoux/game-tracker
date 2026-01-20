use chrono::{DateTime, Duration, Local};
use crate::errors::Error;

fn calculate_end_of_day(day: DateTime<Local>) -> Result<DateTime<Local>, Error> {
    (day + chrono::Duration::days(1))
        .date()
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| Error::CalculateEndOfDayError)
}

#[derive(Debug, Default, Clone)]
pub struct DailyGamingSession {
    start_time: DateTime<Local>,
    end_of_day: DateTime<Local>,
    session_ended: bool,
    duration: Duration,
}

impl DailyGamingSession {
    pub fn new() -> Result<DailyGamingSession, Error> {
        let start_time = Local::now();
        let end_of_day = calculate_end_of_day(start_time)?;

        Ok(
            Self {
                start_time,
                end_of_day,
                session_ended: false,
                duration: Duration::seconds(0),
            }
        )
    }

    pub fn from_duration(duration: Duration) -> Result<DailyGamingSession, Error> {
        let mut session = DailyGamingSession::new()?;
        session.duration = duration;

        Ok(session)
    }

    pub fn should_session_end(&self, time_played: Duration) -> bool {
        self.duration <= time_played
    }

    pub fn day_ended(&self) -> bool {
        self.start_time >= self.end_of_day
    }

    pub fn is_session_ended(&self) -> bool {
        self.session_ended
    }

    pub fn end_session(&mut self) {
        self.session_ended = true;
    }

    pub fn restart_session(&mut self) -> Result<(), Error> {
        self.start_time = Local::now();
        self.session_ended = false;
        self.end_of_day = calculate_end_of_day(self.start_time)?;

        Ok(())
    }

}