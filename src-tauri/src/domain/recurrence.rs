use chrono::{DateTime, Datelike, Local, NaiveDate, TimeZone};

use crate::db::repositories::{RecurrenceConfig, TodoSummary};
use crate::domain::lunar::{next_lunar_yearly_occurrence, solar_ymd_to_lunar_year};

pub fn should_notify_recurrence(
    todo: &TodoSummary,
    now: DateTime<Local>,
    global_advance_hours: u32,
) -> bool {
    let Some(config) = todo.recurrence_json.as_ref() else {
        return false;
    };
    if !config.enabled {
        return false;
    }
    if todo.completed {
        return false;
    }

    let Some(occurrence) = next_occurrence_date(todo, config, now.date_naive()) else {
        return false;
    };

    let Some(moment) = occurrence_at(config, occurrence) else {
        return false;
    };

    let advance_minutes = i64::from(config.advance_minutes) + i64::from(global_advance_hours) * 60;
    let window_start = moment - chrono::Duration::minutes(advance_minutes);
    now >= window_start
}

pub fn recurrence_occurrence_key(todo: &TodoSummary, now: DateTime<Local>) -> Option<String> {
    let config = todo.recurrence_json.as_ref()?;
    if !config.enabled {
        return None;
    }
    let occurrence = next_occurrence_date(todo, config, now.date_naive())?;
    Some(format!("recurrence:{}:{}", todo.id, occurrence))
}

fn next_occurrence_date(
    todo: &TodoSummary,
    config: &RecurrenceConfig,
    from: NaiveDate,
) -> Option<NaiveDate> {
    if config.calendar == "lunar" {
        return next_lunar_occurrence(todo, config, from);
    }
    next_solar_occurrence(todo, config, from)
}

fn next_lunar_occurrence(
    todo: &TodoSummary,
    config: &RecurrenceConfig,
    from: NaiveDate,
) -> Option<NaiveDate> {
    let month = config.lunar_month?;
    let day = config.lunar_day?;
    let leap = config.is_leap_month.unwrap_or(false);
    let interval = config.interval.max(1);
    let start_year = resolve_lunisolar_year(todo, config, from)?;
    next_lunar_yearly_occurrence(start_year, month, day, leap, interval, from)
}

fn resolve_lunisolar_year(
    todo: &TodoSummary,
    config: &RecurrenceConfig,
    from: NaiveDate,
) -> Option<u16> {
    if let Some(first) = config.first_reminder_date.as_deref() {
        if let Some(year) = solar_ymd_to_lunar_year(first) {
            return Some(year);
        }
    }
    let anchor = recurrence_anchor_ymd(todo, config)?;
    if let Some(year) = solar_ymd_to_lunar_year(&anchor) {
        return Some(year);
    }
    chinese_lunisolar_calendar::LunisolarDate::from_date(from)
        .ok()
        .map(|value| value.to_solar_year().to_u16())
}

fn next_solar_occurrence(
    todo: &TodoSummary,
    config: &RecurrenceConfig,
    from: NaiveDate,
) -> Option<NaiveDate> {
    let start = recurrence_start_ymd(todo, config)?;
    let start_date = parse_ymd(&start)?;
    let interval = config.interval.max(1) as i32;
    let mut candidate = start_date;

    for _ in 0..500 {
        if candidate >= from {
            return Some(candidate);
        }
        candidate = advance_solar(candidate, &config.freq, interval)?;
    }
    None
}

fn advance_solar(date: NaiveDate, freq: &str, interval: i32) -> Option<NaiveDate> {
    match freq {
        "daily" => date.checked_add_signed(chrono::Duration::days(i64::from(interval))),
        "weekly" => date.checked_add_signed(chrono::Duration::weeks(i64::from(interval))),
        "monthly" => add_months(date, interval),
        "quarterly" => add_months(date, interval * 3),
        "yearly" => NaiveDate::from_ymd_opt(
            date.year() + interval,
            date.month(),
            date.day(),
        ),
        _ => None,
    }
}

fn add_months(date: NaiveDate, months: i32) -> Option<NaiveDate> {
    let total_months = date.year() * 12 + date.month() as i32 - 1 + months;
    let year = total_months.div_euclid(12);
    let month = (total_months.rem_euclid(12) + 1) as u32;
    let last_day = last_day_of_month(year, month)?;
    let day = date.day().min(last_day);
    NaiveDate::from_ymd_opt(year, month, day)
}

fn last_day_of_month(year: i32, month: u32) -> Option<u32> {
    if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .map(|next| next.pred_opt().map(|d| d.day()))
    .flatten()
}

fn recurrence_start_ymd(todo: &TodoSummary, config: &RecurrenceConfig) -> Option<String> {
    config
        .first_reminder_date
        .clone()
        .or_else(|| recurrence_anchor_ymd(todo, config))
}

fn recurrence_anchor_ymd(todo: &TodoSummary, config: &RecurrenceConfig) -> Option<String> {
    match config.anchor.as_str() {
        "startDate" => todo.start_date.clone(),
        "dueDate" => todo.due_date.clone(),
        _ => todo.due_date.clone(),
    }
}

fn parse_ymd(value: &str) -> Option<NaiveDate> {
    let day_part = value.get(..10).unwrap_or(value);
    NaiveDate::parse_from_str(day_part, "%Y-%m-%d").ok()
}

fn occurrence_at(config: &RecurrenceConfig, date: NaiveDate) -> Option<DateTime<Local>> {
    let (hour, minute) = parse_time(config.time.as_str());
    let naive = date.and_hms_opt(hour, minute, 0)?;
    Local.from_local_datetime(&naive).single()
}

fn parse_time(value: &str) -> (u32, u32) {
    let mut parts = value.split(':');
    let hour = parts.next().and_then(|v| v.parse().ok()).unwrap_or(9);
    let minute = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0);
    (hour, minute)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lunar_birthday_maps_to_solar() {
        let date = lunar_to_naive_date(2024, 3, 5, false).expect("lunar date");
        assert_eq!(date.to_string(), "2024-04-13");
    }
}
