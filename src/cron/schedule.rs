use crate::cron::Schedule;
use anyhow::{Context, Result};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use cron::Schedule as CronExprSchedule;
use interim::{parse_duration, Interval};
use std::str::FromStr;

pub fn next_run_for_schedule(schedule: &Schedule, from: DateTime<Utc>) -> Result<DateTime<Utc>> {
    match schedule {
        Schedule::Cron { expr, tz } => {
            let normalized = normalize_expression(expr)?;
            let cron = CronExprSchedule::from_str(&normalized)
                .with_context(|| format!("Invalid cron expression: {expr}"))?;

            if let Some(tz_name) = tz {
                let timezone = chrono_tz::Tz::from_str(tz_name)
                    .with_context(|| format!("Invalid IANA timezone: {tz_name}"))?;
                let localized_from = from.with_timezone(&timezone);
                let next_local = cron.after(&localized_from).next().ok_or_else(|| {
                    anyhow::anyhow!("No future occurrence for expression: {expr}")
                })?;
                Ok(next_local.with_timezone(&Utc))
            } else {
                cron.after(&from)
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("No future occurrence for expression: {expr}"))
            }
        }
        Schedule::At { at } => Ok(*at),
        Schedule::In { in_ } => apply_delay_to_datetime(in_, from),
        Schedule::Every { every_ms } => {
            if *every_ms == 0 {
                anyhow::bail!("Invalid schedule: every_ms must be > 0");
            }
            let ms = i64::try_from(*every_ms).context("every_ms is too large")?;
            let delta = ChronoDuration::milliseconds(ms);
            from.checked_add_signed(delta)
                .ok_or_else(|| anyhow::anyhow!("every_ms overflowed DateTime"))
        }
    }
}

fn apply_delay_to_datetime(delay: &str, from: DateTime<Utc>) -> Result<DateTime<Utc>> {
    let interval =
        parse_duration(delay).map_err(|e| anyhow::anyhow!("Invalid delay '{}': {}", delay, e))?;

    match interval {
        Interval::Seconds(secs) => from
            .checked_add_signed(ChronoDuration::seconds(secs.into()))
            .ok_or_else(|| anyhow::anyhow!("Delay '{}' overflowed DateTime", delay)),
        Interval::Days(days) => from
            .checked_add_signed(ChronoDuration::days(days.into()))
            .ok_or_else(|| anyhow::anyhow!("Delay '{}' overflowed DateTime", delay)),
        Interval::Months(months) => from
            .checked_add_months(chrono::Months::new(months.unsigned_abs() as u32))
            .ok_or_else(|| anyhow::anyhow!("Delay '{}' overflowed DateTime", delay)),
    }
}

pub fn validate_schedule(schedule: &Schedule, now: DateTime<Utc>) -> Result<()> {
    match schedule {
        Schedule::Cron { expr, .. } => {
            let _ = normalize_expression(expr)?;
            let _ = next_run_for_schedule(schedule, now)?;
            Ok(())
        }
        Schedule::At { at } => {
            if *at <= now {
                anyhow::bail!("Invalid schedule: 'at' must be in the future");
            }
            Ok(())
        }
        Schedule::In { in_ } => {
            let _ = parse_duration(in_)
                .map_err(|e| anyhow::anyhow!("Invalid delay '{}': {}", in_, e))?;
            Ok(())
        }
        Schedule::Every { every_ms } => {
            if *every_ms == 0 {
                anyhow::bail!("Invalid schedule: every_ms must be > 0");
            }
            Ok(())
        }
    }
}

pub fn schedule_cron_expression(schedule: &Schedule) -> Option<String> {
    match schedule {
        Schedule::Cron { expr, .. } => Some(expr.clone()),
        _ => None,
    }
}

pub fn normalize_expression(expression: &str) -> Result<String> {
    let expression = expression.trim();
    let field_count = expression.split_whitespace().count();

    match field_count {
        // standard crontab syntax: minute hour day month weekday
        5 => Ok(format!("0 {expression}")),
        // crate-native syntax includes seconds (+ optional year)
        6 | 7 => Ok(expression.to_string()),
        _ => anyhow::bail!(
            "Invalid cron expression: {expression} (expected 5, 6, or 7 fields, got {field_count})"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn next_run_for_schedule_supports_every_and_at() {
        let now = Utc::now();
        let every = Schedule::Every { every_ms: 60_000 };
        let next = next_run_for_schedule(&every, now).unwrap();
        assert!(next > now);

        let at = now + ChronoDuration::minutes(10);
        let at_schedule = Schedule::At { at };
        let next_at = next_run_for_schedule(&at_schedule, now).unwrap();
        assert_eq!(next_at, at);
    }

    #[test]
    fn next_run_for_schedule_supports_in_delay_minutes() {
        let now = Utc::now();
        let in_schedule = Schedule::In {
            in_: "5 minutes".to_string(),
        };
        let next = next_run_for_schedule(&in_schedule, now).unwrap();
        let expected = now + ChronoDuration::minutes(5);
        assert!(next > now);
        assert!((next - expected).num_seconds().abs() < 1);
    }

    #[test]
    fn next_run_for_schedule_supports_in_delay_hours() {
        let now = Utc::now();
        let in_schedule = Schedule::In {
            in_: "2 hours".to_string(),
        };
        let next = next_run_for_schedule(&in_schedule, now).unwrap();
        let expected = now + ChronoDuration::hours(2);
        assert!((next - expected).num_seconds().abs() < 1);
    }

    #[test]
    fn next_run_for_schedule_supports_in_delay_days() {
        let now = Utc::now();
        let in_schedule = Schedule::In {
            in_: "3 days".to_string(),
        };
        let next = next_run_for_schedule(&in_schedule, now).unwrap();
        let expected = now + ChronoDuration::days(3);
        assert!((next - expected).num_seconds().abs() < 1);
    }

    #[test]
    fn next_run_for_schedule_supports_in_delay_months() {
        let from = Utc.with_ymd_and_hms(2026, 1, 31, 12, 0, 0).unwrap();
        let in_schedule = Schedule::In {
            in_: "1 month".to_string(),
        };
        let next = next_run_for_schedule(&in_schedule, from).unwrap();
        // Jan 31 + 1 month = Feb 28 (or 29 in leap year)
        assert_eq!(next.month(), 2);
        assert_eq!(next.day(), 28);
    }

    #[test]
    fn validate_schedule_rejects_invalid_in_delay() {
        let now = Utc::now();
        let schedule = Schedule::In {
            in_: "invalid".to_string(),
        };
        assert!(validate_schedule(&schedule, now).is_err());
    }

    #[test]
    fn next_run_for_schedule_supports_timezone() {
        let from = Utc.with_ymd_and_hms(2026, 2, 16, 0, 0, 0).unwrap();
        let schedule = Schedule::Cron {
            expr: "0 9 * * *".into(),
            tz: Some("America/Los_Angeles".into()),
        };

        let next = next_run_for_schedule(&schedule, from).unwrap();
        assert_eq!(next, Utc.with_ymd_and_hms(2026, 2, 16, 17, 0, 0).unwrap());
    }
}
