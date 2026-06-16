use chrono::NaiveDate;
use chinese_lunisolar_calendar::LunisolarDate;

pub fn lunar_to_naive_date(
    lunisolar_year: u16,
    month: u8,
    day: u8,
    leap: bool,
) -> Option<NaiveDate> {
    let lunisolar = LunisolarDate::from_ymd(lunisolar_year, month, leap, day).ok()?;
    Some(lunisolar.to_naive_date())
}

pub fn solar_ymd_to_lunar_year(solar_ymd: &str) -> Option<u16> {
    let date = parse_ymd(solar_ymd)?;
    let lunisolar = LunisolarDate::from_date(date).ok()?;
    Some(lunisolar.to_solar_year().to_u16())
}

fn parse_ymd(value: &str) -> Option<NaiveDate> {
    let day_part = value.get(..10).unwrap_or(value);
    NaiveDate::parse_from_str(day_part, "%Y-%m-%d").ok()
}

pub fn next_lunar_yearly_occurrence(
    lunisolar_year_start: u16,
    month: u8,
    day: u8,
    leap: bool,
    interval: u32,
    from: NaiveDate,
) -> Option<NaiveDate> {
    let mut year = lunisolar_year_start;
    let max_year = lunisolar_year_start.saturating_add(200);
    while year <= max_year {
        if let Some(candidate) = lunar_to_naive_date(year, month, day, leap) {
            if candidate >= from {
                return Some(candidate);
            }
        }
        year = year.saturating_add(interval as u16);
    }
    None
}
