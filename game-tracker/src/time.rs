use std::str::FromStr;
use chrono::TimeDelta;
use regex::{Matches, Regex, RegexSet};
use crate::errors::Error;

#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
pub struct DurationParser {
    hours: u64,
    minutes: u64,
    seconds: u64,
}

fn parse_hms_duration(extractor: Matches, session_duration: &mut DurationParser) -> Result<(), Error> {
    for current_match in extractor {
        let mut base_str = current_match.as_str().to_string();
        let time_modifier = base_str.pop()
            .ok_or(Error::SessionDurationParserError)?;

        match time_modifier {
            'h'|'H' => session_duration.hours += u64::from_str(base_str.as_str())?,
            'm'|'M' => session_duration.minutes += u64::from_str(base_str.as_str())?,
            's'|'S' => session_duration.seconds += u64::from_str(base_str.as_str())?,
            _ => return Err(Error::SessionDurationParserError),
        }
    }

    Ok(())
}

fn parse_colon_duration(time: &str, session_duration: &mut DurationParser) -> Result<(), Error> {
    let mut parts = time.splitn(3, ":")
        .collect::<Vec<&str>>();

    for (index, value) in parts.iter_mut().enumerate() {
        let parsed_value = value.parse::<u64>()?;
        match index {
            0 => session_duration.hours += parsed_value,
            1 => session_duration.minutes += parsed_value,
            2 => session_duration.seconds += parsed_value,
            _ => return Err(Error::SessionDurationParserError)
        }
    }

    Ok(())
}


impl FromStr for DurationParser {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut session_duration = DurationParser::default();
        let re = RegexSet::new([
            r"(\d+[hHmMsS]\s?)+",
            r"^\d+:\d+:\d+$",
        ])?;


        for i in re.matches(s) {
            let current_re = re.patterns()[i].clone();
            match i {
                0 => {
                    let extractor = Regex::new(current_re.as_str())?;
                    parse_hms_duration(extractor.find_iter(s), &mut session_duration)?;
                },
                1 => parse_colon_duration(s, &mut session_duration)?,
                _ => return Err(Error::SessionDurationParserError),
            }
        }
        Ok(session_duration)
    }
}

impl DurationParser {

    pub fn to_seconds(&self) -> u64 {
        (self.hours * 60 * 60) + (self.minutes * 60) + self.seconds
    }

    pub fn to_string(&self) -> String {
        format!("{} hour(s) {} minute(s) {} second(s)",
                self.hours, self.minutes, self.seconds
        )
    }

}

pub fn format_duration(duration: u64) -> String {
    let delta = TimeDelta::new(duration as i64, 0)
        .expect("could not convert to time delta");

    let days = delta.num_days();
    let hours = delta.num_hours() - (24 * delta.num_days());
    let minutes = delta.num_minutes() - (delta.num_hours() * 60);
    let seconds = delta.num_seconds() - (delta.num_minutes() * 60);

    format!("{} days {} hour(s) {} minute(s) {} second(s)",
            days, hours, minutes, seconds
    )
}

#[cfg(test)]
mod session_duration_parser_tests {
    use super::*;

    #[test]
    fn test_first_case() {
        let a = DurationParser::from_str("1000h 30m 9000s")
            .expect("no errors!");
        assert_eq!(a.hours, 1000);
        assert_eq!(a.minutes, 30);
        assert_eq!(a.seconds, 9000);
    }

    #[test]
    fn test_first_case_with_duplicates() {
        let a = DurationParser::from_str("1000h 10h 30m 30m 10s 9000s")
            .expect("no errors!");

        assert_eq!(a.hours, 1010);
        assert_eq!(a.minutes, 60);
        assert_eq!(a.seconds, 9010);
    }

    #[test]
    fn test_first_case_with_invalid_values() {
        DurationParser::from_str("102h avasds")
            .expect_err("should not work!");

        DurationParser::from_str("102h 83223")
            .expect_err("should not work!");
    }
}