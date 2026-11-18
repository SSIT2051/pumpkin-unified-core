use std::time::{SystemTime, UNIX_EPOCH};

/// Current UNIX timestamp in seconds.
pub fn now_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Parses a duration string such as `1h30m`, `2d`, `45s`, `10m` into seconds.
/// Returns `None` if the string is empty, malformed, or evaluates to zero.
pub fn parse_duration_seconds(input: &str) -> Option<i64> {
    let input = input.trim().to_lowercase();
    if input.is_empty() {
        return None;
    }

    let mut total: i64 = 0;
    let mut num = String::new();
    let mut found = false;

    for c in input.chars() {
        if c.is_ascii_digit() {
            num.push(c);
        } else {
            if num.is_empty() {
                return None;
            }
            let n: i64 = num.parse().ok()?;
            let mult = match c {
                'd' => 86_400,
                'h' => 3_600,
                'm' => 60,
                's' => 1,
                _ => return None,
            };
            total += n * mult;
            num.clear();
            found = true;
        }
    }

    // Trailing number without a unit is treated as seconds.
    if !num.is_empty() {
        let n: i64 = num.parse().ok()?;
        total += n;
        found = true;
    }

    if found && total > 0 { Some(total) } else { None }
}

/// Formats a number of seconds into a compact `1d 2h 3m 4s` string.
pub fn format_duration(total_seconds: i64) -> String {
    let mut parts = Vec::new();
    let days = total_seconds / 86_400;
    if days > 0 {
        parts.push(format!("{days}d"));
    }
    let hours = (total_seconds % 86_400) / 3_600;
    if hours > 0 {
        parts.push(format!("{hours}h"));
    }
    let minutes = (total_seconds % 3_600) / 60;
    if minutes > 0 {
        parts.push(format!("{minutes}m"));
    }
    let seconds = total_seconds % 60;
    if seconds > 0 || parts.is_empty() {
        parts.push(format!("{seconds}s"));
    }
    parts.join(" ")
}

/// Returns `true` if `timestamp + duration_seconds` is in the past.
pub fn is_expired(timestamp: i64, duration_seconds: i64) -> bool {
    now_timestamp() > timestamp + duration_seconds
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_compound_durations() {
        assert_eq!(parse_duration_seconds("1h30m"), Some(5_400));
        assert_eq!(parse_duration_seconds("2d"), Some(172_800));
        assert_eq!(parse_duration_seconds("45"), Some(45));
    }

    #[test]
    fn rejects_invalid_durations() {
        assert_eq!(parse_duration_seconds(""), None);
        assert_eq!(parse_duration_seconds("abc"), None);
        assert_eq!(parse_duration_seconds("0s"), None);
    }
}