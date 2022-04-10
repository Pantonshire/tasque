use std::cmp::Ordering;
use std::num::NonZeroU8;

use chrono::{DateTime, Datelike, Duration, NaiveDate, Timelike};

use crate::time_val::TimeVal;
use crate::TimeZoneExt;

const MIN_DAYS: u8 = 28;

pub struct Builder {
    day: TimeVal<31>,
    hour: TimeVal<24>,
    minute: TimeVal<60>,
    second: TimeVal<60>,
}

impl Builder {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            day: TimeVal::Every,
            hour: TimeVal::Every,
            minute: TimeVal::Every,
            second: TimeVal::Every,
        }
    }

    #[must_use]
    pub const fn build(self) -> TaskTime {
        TaskTime::new(self.day, self.hour, self.minute, self.second)
    }

    #[must_use]
    pub const fn at_day(self, day: NonZeroU8) -> Self {
        Self {
            day: TimeVal::at(day.get() - 1),
            ..self
        }
    }

    #[must_use]
    pub const fn every_day(self) -> Self {
        Self {
            day: TimeVal::Every,
            ..self
        }
    }

    #[must_use]
    pub const fn at_hour(self, hour: u8) -> Self {
        Self {
            hour: TimeVal::at(hour),
            ..self
        }
    }

    #[must_use]
    pub const fn every_hour(self) -> Self {
        Self {
            hour: TimeVal::Every,
            ..self
        }
    }

    #[must_use]
    pub const fn at_minute(self, minute: u8) -> Self {
        Self {
            minute: TimeVal::at(minute),
            ..self
        }
    }

    #[must_use]
    pub const fn every_minute(self) -> Self {
        Self {
            minute: TimeVal::Every,
            ..self
        }
    }

    #[must_use]
    pub const fn at_second(self, second: u8) -> Self {
        Self {
            second: TimeVal::at(second),
            ..self
        }
    }

    #[must_use]
    pub const fn every_second(self) -> Self {
        Self {
            second: TimeVal::Every,
            ..self
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct TaskTime {
    day: TimeVal<31>,
    hour: TimeVal<24>,
    minute: TimeVal<60>,
    second: TimeVal<60>,
}

impl TaskTime {
    pub const EVERY_DAY: Self = Self::builder()
        .every_day()
        .at_hour(0)
        .at_minute(0)
        .at_second(0)
        .build();

    pub const EVERY_HOUR: Self = Self::builder()
        .every_day()
        .every_hour()
        .at_minute(0)
        .at_second(0)
        .build();

    pub const EVERY_MINUTE: Self = Self::builder()
        .every_day()
        .every_hour()
        .every_minute()
        .at_second(0)
        .build();

    #[must_use]
    pub const fn builder() -> Builder {
        Builder::new()
    }

    #[must_use]
    pub const fn new(
        day: TimeVal<31>,
        hour: TimeVal<24>,
        minute: TimeVal<60>,
        second: TimeVal<60>,
    ) -> Self {
        Self {
            day,
            hour,
            minute,
            second,
        }
    }

    pub(crate) fn next_occurrence<Tz>(self, now: DateTime<Tz>) -> DateTime<Tz>
    where
        Tz: TimeZoneExt,
        Tz::Offset: Copy,
    {
        self.advance_to_dhms(now)
    }

    fn advance_to_dhms<Tz>(self, time: DateTime<Tz>) -> DateTime<Tz>
    where
        Tz: TimeZoneExt,
        Tz::Offset: Copy,
    {
        // First, advance the time to a time which satisfies the hour, minute and second
        // requirements
        let time = self.advance_to_hms(time);

        let target_day: u8;
        let day_less_than_target: bool;

        match self.day {
            TimeVal::At(day) => {
                let day = day.get();
                match time.day0().cmp(&u32::from(day)) {
                    Ordering::Equal => return time,
                    Ordering::Less => {
                        target_day = day;
                        day_less_than_target = true;
                    }
                    Ordering::Greater => {
                        target_day = day;
                        day_less_than_target = false;
                    }
                }
            }
            TimeVal::Every => return time,
        };

        let time = time
            .with_hour(u32::from(self.hour.min_valid()))
            .and_then(|time| time.with_minute(u32::from(self.minute.min_valid())))
            .and_then(|time| time.with_second(u32::from(self.second.min_valid())))
            .expect("invalid time");

        // If the current day is less than the target day and the target day does not exceed the
        // number of days in the current month, then simply set the day to the target day without
        // changing anything else
        if day_less_than_target
            && (target_day < MIN_DAYS
                || i64::from(target_day) < days_in_month(time.year(), time.month()))
        {
            return time.with_day0(u32::from(target_day)).expect("invalid time");
        }

        let (mut year, mut month) = (time.year(), time.month());

        loop {
            // Advance to the next month
            (year, month) = match month {
                12 => (year + 1, 1),
                _ => (year, month + 1),
            };

            if target_day < MIN_DAYS || i64::from(target_day) < days_in_month(year, month) {
                // We cannot update the month and day one-at-a-time like we can with hours /
                // minutes / seconds because this may cause an intermediate value which is an
                // invalid `DateTime`. Therefore, we use `TimeZone::ymd` instead to set the month
                // and day at the same time.
                return time
                    .timezone()
                    .ymd(year, month, u32::from(target_day) + 1)
                    .and_hms(time.hour(), time.minute(), time.second());
            }
        }
    }

    fn advance_to_hms<Tz>(self, time: DateTime<Tz>) -> DateTime<Tz>
    where
        Tz: TimeZoneExt,
        Tz::Offset: Copy,
    {
        let time = self.advance_to_ms(time);
        match self.hour {
            TimeVal::At(hour) => {
                let hour = hour.get();
                match time.hour().cmp(&u32::from(hour)) {
                    Ordering::Equal => time,
                    Ordering::Less => time
                        .with_hour(u32::from(hour))
                        .and_then(|time| time.with_minute(u32::from(self.minute.min_valid())))
                        .and_then(|time| time.with_second(u32::from(self.second.min_valid())))
                        .expect("invalid time"),
                    Ordering::Greater => time
                        .date()
                        .succ()
                        .and_hms(u32::from(hour), time.minute(), time.second())
                        .with_minute(u32::from(self.minute.min_valid()))
                        .and_then(|time| time.with_second(u32::from(self.second.min_valid())))
                        .expect("invalid time"),
                }
            }
            TimeVal::Every => time,
        }
    }

    fn advance_to_ms<Tz>(self, time: DateTime<Tz>) -> DateTime<Tz>
    where
        Tz: TimeZoneExt,
        Tz::Offset: Copy,
    {
        let time = self.advance_to_s(time);
        match self.minute {
            TimeVal::At(minute) => {
                let minute = minute.get();
                match time.minute().cmp(&u32::from(minute)) {
                    Ordering::Equal => time,
                    Ordering::Less => time
                        .with_minute(u32::from(minute))
                        .and_then(|time| time.with_second(u32::from(self.second.min_valid())))
                        .expect("invalid time"),
                    Ordering::Greater => (time + Duration::hours(1))
                        .with_minute(u32::from(minute))
                        .and_then(|time| time.with_second(u32::from(self.second.min_valid())))
                        .expect("invalid time"),
                }
            }
            TimeVal::Every => time,
        }
    }

    fn advance_to_s<Tz>(self, time: DateTime<Tz>) -> DateTime<Tz>
    where
        Tz: TimeZoneExt,
        Tz::Offset: Copy,
    {
        match self.second {
            TimeVal::At(second) => {
                let second = second.get();
                match time.second().cmp(&u32::from(second)) {
                    Ordering::Equal => time,
                    Ordering::Less => time.with_second(u32::from(second)).expect("invalid time"),
                    Ordering::Greater => (time + Duration::minutes(1))
                        .with_second(u32::from(second))
                        .expect("invalid time"),
                }
            }
            TimeVal::Every => time,
        }
    }
}

fn days_in_month(year: i32, month: u32) -> i64 {
    let month_start = NaiveDate::from_ymd(year, month, 1);
    let next_month_start = match month {
        12 => NaiveDate::from_ymd(year + 1, 1, 1),
        _ => NaiveDate::from_ymd(year, month + 1, 1),
    };

    next_month_start
        .signed_duration_since(month_start)
        .num_days()
}

#[cfg(test)]
#[allow(clippy::pedantic)]
mod tests {
    use super::TaskTime;
    use chrono::{TimeZone, Utc};
    use std::num::NonZeroU8;

    #[test]
    fn test_next_occurrence() {
        assert_eq!(
            TaskTime::builder()
                .at_hour(11)
                .at_minute(16)
                .at_second(41)
                .build()
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)),
            Utc.ymd(2022, 4, 5).and_hms(11, 16, 41)
        );

        assert_eq!(
            TaskTime::builder()
                .at_hour(18)
                .at_minute(16)
                .at_second(41)
                .build()
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)),
            Utc.ymd(2022, 4, 4).and_hms(18, 16, 41)
        );

        assert_eq!(
            TaskTime::builder()
                .at_hour(11)
                .at_minute(16)
                .every_second()
                .build()
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)),
            Utc.ymd(2022, 4, 5).and_hms(11, 16, 0)
        );

        assert_eq!(
            TaskTime::builder()
                .at_hour(11)
                .every_minute()
                .at_second(14)
                .build()
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)),
            Utc.ymd(2022, 4, 5).and_hms(11, 0, 14)
        );

        assert_eq!(
            TaskTime::builder()
                .at_hour(11)
                .every_minute()
                .every_second()
                .build()
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)),
            Utc.ymd(2022, 4, 5).and_hms(11, 0, 0)
        );

        assert_eq!(
            TaskTime::builder()
                .at_hour(18)
                .every_minute()
                .every_second()
                .build()
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)),
            Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)
        );

        assert_eq!(
            TaskTime::builder()
                .at_hour(19)
                .every_minute()
                .every_second()
                .build()
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)),
            Utc.ymd(2022, 4, 4).and_hms(19, 0, 0)
        );

        assert_eq!(
            TaskTime::builder()
                .every_hour()
                .every_minute()
                .every_second()
                .build()
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)),
            Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)
        );

        assert_eq!(
            TaskTime::builder()
                .every_hour()
                .at_minute(0)
                .at_second(0)
                .build()
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)),
            Utc.ymd(2022, 4, 4).and_hms(19, 0, 0)
        );

        assert_eq!(
            TaskTime::builder()
                .at_hour(11)
                .at_minute(16)
                .at_second(41)
                .build()
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(11, 16, 40)),
            Utc.ymd(2022, 4, 4).and_hms(11, 16, 41)
        );

        assert_eq!(
            TaskTime::builder()
                .at_hour(11)
                .at_minute(16)
                .at_second(41)
                .build()
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(11, 16, 41)),
            Utc.ymd(2022, 4, 4).and_hms(11, 16, 41)
        );

        assert_eq!(
            TaskTime::builder()
                .at_hour(11)
                .at_minute(16)
                .at_second(41)
                .build()
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(11, 16, 42)),
            Utc.ymd(2022, 4, 5).and_hms(11, 16, 41)
        );

        assert_eq!(
            TaskTime::builder()
                .at_day(NonZeroU8::new(5).unwrap())
                .at_hour(11)
                .at_minute(16)
                .at_second(41)
                .build()
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)),
            Utc.ymd(2022, 4, 5).and_hms(11, 16, 41)
        );

        assert_eq!(
            TaskTime::builder()
                .at_day(NonZeroU8::new(4).unwrap())
                .at_hour(11)
                .at_minute(16)
                .at_second(41)
                .build()
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)),
            Utc.ymd(2022, 5, 4).and_hms(11, 16, 41)
        );

        assert_eq!(
            TaskTime::builder()
                .at_day(NonZeroU8::new(29).unwrap())
                .at_hour(11)
                .at_minute(16)
                .at_second(41)
                .build()
                .next_occurrence(Utc.ymd(2022, 2, 10).and_hms(18, 1, 14)),
            Utc.ymd(2022, 3, 29).and_hms(11, 16, 41)
        );

        assert_eq!(
            TaskTime::builder()
                .at_day(NonZeroU8::new(29).unwrap())
                .at_hour(11)
                .at_minute(16)
                .at_second(41)
                .build()
                .next_occurrence(Utc.ymd(2024, 2, 10).and_hms(18, 1, 14)),
            Utc.ymd(2024, 2, 29).and_hms(11, 16, 41)
        );

        assert_eq!(
            TaskTime::builder()
                .at_day(NonZeroU8::new(31).unwrap())
                .at_hour(11)
                .at_minute(16)
                .at_second(41)
                .build()
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)),
            Utc.ymd(2022, 5, 31).and_hms(11, 16, 41)
        );

        assert_eq!(
            TaskTime::builder()
                .at_day(NonZeroU8::new(31).unwrap())
                .at_hour(11)
                .at_minute(16)
                .at_second(41)
                .build()
                .next_occurrence(Utc.ymd(2022, 3, 4).and_hms(18, 1, 14)),
            Utc.ymd(2022, 3, 31).and_hms(11, 16, 41)
        );

        assert_eq!(
            TaskTime::builder()
                .at_day(NonZeroU8::new(4).unwrap())
                .at_hour(11)
                .at_minute(16)
                .at_second(41)
                .build()
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(9, 13, 20)),
            Utc.ymd(2022, 4, 4).and_hms(11, 16, 41)
        );

        assert_eq!(
            TaskTime::builder()
                .at_day(NonZeroU8::new(1).unwrap())
                .every_hour()
                .every_minute()
                .every_second()
                .build()
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)),
            Utc.ymd(2022, 5, 1).and_hms(0, 0, 0)
        );

        assert_eq!(
            TaskTime::builder()
                .at_day(NonZeroU8::new(4).unwrap())
                .every_hour()
                .every_minute()
                .every_second()
                .build()
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)),
            Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)
        );

        assert_eq!(
            TaskTime::builder()
                .at_day(NonZeroU8::new(5).unwrap())
                .every_hour()
                .every_minute()
                .every_second()
                .build()
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)),
            Utc.ymd(2022, 4, 5).and_hms(0, 0, 0)
        );

        assert_eq!(
            TaskTime::builder()
                .at_day(NonZeroU8::new(1).unwrap())
                .every_hour()
                .at_minute(30)
                .every_second()
                .build()
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)),
            Utc.ymd(2022, 5, 1).and_hms(0, 30, 0)
        );

        assert_eq!(
            TaskTime::builder()
                .at_day(NonZeroU8::new(9).unwrap())
                .every_hour()
                .every_minute()
                .every_second()
                .build()
                .next_occurrence(Utc.ymd(2022, 5, 31).and_hms(18, 1, 14)),
            Utc.ymd(2022, 6, 9).and_hms(0, 0, 0)
        );
    }
}
