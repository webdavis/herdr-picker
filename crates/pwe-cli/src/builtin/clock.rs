use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

// Seconds east of UTC, as `date +%z` says, so times show the way the Mac's clock does.
pub(super) fn local_offset() -> i64 {
    let said = Command::new("date").arg("+%z").output();
    said.ok()
        .and_then(|said| offset(String::from_utf8_lossy(&said.stdout).trim()))
        .unwrap_or(0)
}

pub(super) fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs() as i64)
}

fn offset(zone: &str) -> Option<i64> {
    let (sign, digits) = zone.split_at_checked(1)?;
    let hours: i64 = digits.get(..2)?.parse().ok()?;
    let minutes: i64 = digits.get(2..4)?.parse().ok()?;
    let seconds = hours * 3600 + minutes * 60;
    match sign {
        "+" => Some(seconds),
        "-" => Some(-seconds),
        _ => None,
    }
}

// A moment as the pickers show one: `2026-10-19 06:07`.
pub(super) fn stamp(epoch: i64, offset: i64) -> String {
    let local = epoch + offset;
    let (year, month, day) = civil(local.div_euclid(86_400));
    let minute = local.rem_euclid(86_400) / 60;
    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02}",
        minute / 60,
        minute % 60
    )
}

// The moment an ISO 8601 time names, like `2026-10-19T12:07:44Z` or one with a `+02:00` offset.
pub(super) fn parse_iso(text: &str) -> Option<i64> {
    let number = |range: std::ops::Range<usize>| text.get(range)?.parse::<i64>().ok();
    let days = days(number(0..4)?, number(5..7)?, number(8..10)?);
    let seconds = number(11..13)? * 3600 + number(14..16)? * 60 + number(17..19)?;
    let zone = text[19..].trim_start_matches(|c: char| c == '.' || c.is_ascii_digit());
    let east = match zone {
        "Z" | "" => 0,
        zone => offset(&zone.replace(':', ""))?,
    };
    Some(days * 86_400 + seconds - east)
}

// Howard Hinnant's days-from-civil and its inverse, counted from 1970-01-01.
fn days(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let of_era = year - era * 400;
    let of_year = (153 * ((month + 9) % 12) + 2) / 5 + day - 1;
    let of_cycle = of_era * 365 + of_era / 4 - of_era / 100 + of_year;
    era * 146_097 + of_cycle - 719_468
}

fn civil(days: i64) -> (i64, i64, i64) {
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let of_era = shifted - era * 146_097;
    let of_cycle = (of_era - of_era / 1460 + of_era / 36_524 - of_era / 146_096) / 365;
    let of_year = of_era - (365 * of_cycle + of_cycle / 4 - of_cycle / 100);
    let shifted_month = (5 * of_year + 2) / 153;
    let day = of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    let year = of_cycle + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

#[cfg(test)]
mod tests;
