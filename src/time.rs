use chrono::TimeDelta;

// todo : implement parser -> 3h\s?43m\s?21s / 08:30:12

pub fn to_seconds(hours: u64, minutes: u64, seconds: u64) -> u64 {
    (hours * 60 * 60) + (minutes * 60) + seconds
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