mod component;

use std::error;
use std::fmt;
use std::num::NonZeroU8;
use std::ops::RangeInclusive;

use chrono::{DateTime, Datelike, Duration, NaiveDate, Timelike};

use crate::TimeZoneExt;
use component::Component;

const MIN_DAYS: u8 = 28;

#[derive(Clone, Copy, Default, Debug)]
pub struct Schedule {
    day: Component<30>,
    hour: Component<23>,
    minute: Component<59>,
    second: Component<59>,
}

impl Schedule {
    #[must_use]
    pub(crate) fn new(
        day: Component<30>,
        hour: Component<23>,
        minute: Component<59>,
        second: Component<59>,
    ) -> Self {
        Self {
            day,
            hour,
            minute,
            second,
        }
    }

    #[must_use]
    pub fn new_every_second() -> Self {
        Self::new(
            Component::every(),
            Component::every(),
            Component::every(),
            Component::every()
        )
    }

    #[must_use]
    pub fn new_every_minute() -> Self {
        Self::new(
            Component::every(),
            Component::every(),
            Component::every(),
            Component::exactly_zero(),
        )
    }

    #[must_use]
    pub fn new_every_hour() -> Self {
        Self::new(
            Component::every(),
            Component::every(),
            Component::exactly_zero(),
            Component::exactly_zero(),
        )
    }

    #[must_use]
    pub fn new_every_day() -> Self {
        Self::new(
            Component::every(),
            Component::exactly_zero(),
            Component::exactly_zero(),
            Component::exactly_zero(),
        )
    }

    #[must_use]
    pub fn new_every_month() -> Self {
        Self::new(
            Component::exactly_zero(),
            Component::exactly_zero(),
            Component::exactly_zero(),
            Component::exactly_zero(),
        )
    }

    #[must_use]
    pub fn at_day(self, day: NonZeroU8) -> Self {
        Self {
            day: match Component::exactly((day.get() - 1) % 31) {
                Ok(day) => day,
                Err(_) => unreachable!(),
            },
            ..self
        }
    }

    #[must_use]
    pub fn at_first_day(self) -> Self {
        Self { day: Component::exactly_zero(), ..self }
    }

    #[must_use]
    pub fn at_every_day(self) -> Self {
        Self { day: Component::every(), ..self }
    }

    #[must_use]
    pub fn at_every_nth_day(self, n: NonZeroU8) -> Self {
        Self { day: Component::every_step(n), ..self }
    }

    fn day1_range_to_day0_range(range: RangeInclusive<NonZeroU8>) -> (u8, u8) {
        (range.start().get() - 1, range.end().get() - 1)
    }

    pub fn at_every_day_between(self, range: RangeInclusive<NonZeroU8>) -> Result<Self, Error> {
        let (start, end) = Self::day1_range_to_day0_range(range);
        Ok(Self { day: Component::between(start, end).map_err(|_| Error)?, ..self })
    }

    pub fn at_every_nth_day_between(
        self,
        range: RangeInclusive<NonZeroU8>,
        n: NonZeroU8
    ) -> Result<Self, Error>
    {
        let (start, end) = Self::day1_range_to_day0_range(range);
        Ok(Self { day: Component::new(start, end, n).map_err(|_| Error)?, ..self })
    }

    #[must_use]
    pub fn at_hour(self, hour: u8) -> Self {
        Self {
            hour: match Component::exactly(hour % 24) {
                Ok(hour) => hour,
                Err(_) => unreachable!(),
            },
            ..self
        }
    }

    #[must_use]
    pub fn at_zero_hour(self) -> Self {
        Self { hour: Component::exactly_zero(), ..self }
    }

    #[must_use]
    pub fn at_every_hour(self) -> Self {
        Self { hour: Component::every(), ..self }
    }

    #[must_use]
    pub fn at_every_nth_hour(self, n: NonZeroU8) -> Self {
        Self { hour: Component::every_step(n), ..self }
    }

    pub fn at_every_hour_between(self, range: RangeInclusive<u8>) -> Result<Self, Error> {
        Ok(Self {
            hour: Component::between(*range.start(), *range.end()).map_err(|_| Error)?,
            ..self
        })
    }

    pub fn at_every_nth_hour_between(
        self,
        range: RangeInclusive<u8>,
        n: NonZeroU8
    ) -> Result<Self, Error>
    {
        Ok(Self {
            hour: Component::new(*range.start(), *range.end(), n).map_err(|_| Error)?,
            ..self
        })
    }

    #[must_use]
    pub fn at_minute(self, minute: u8) -> Self {
        Self {
            minute: match Component::exactly(minute % 60) {
                Ok(minute) => minute,
                Err(_) => unreachable!(),
            },
            ..self
        }
    }

    #[must_use]
    pub fn at_zero_minute(self) -> Self {
        Self { minute: Component::exactly_zero(), ..self }
    }

    #[must_use]
    pub fn at_every_minute(self) -> Self {
        Self { minute: Component::every(), ..self }
    }

    #[must_use]
    pub fn at_every_nth_minute(self, n: NonZeroU8) -> Self {
        Self { minute: Component::every_step(n), ..self }
    }

    pub fn at_every_minute_between(self, range: RangeInclusive<u8>) -> Result<Self, Error> {
        Ok(Self { minute: Component::between(*range.start(), *range.end()).map_err(|_| Error)?, ..self })
    }

    pub fn at_every_nth_minute_between(
        self,
        range: RangeInclusive<u8>,
        n: NonZeroU8
    ) -> Result<Self, Error>
    {
        Ok(Self { minute: Component::new(*range.start(), *range.end(), n).map_err(|_| Error)?, ..self })
    }

    #[must_use]
    pub fn at_second(self, second: u8) -> Self {
        Self { 
            second: match Component::exactly(second % 60) {
                Ok(second) => second,
                Err(_) => unreachable!(),
            },
            ..self
        }
    }

    #[must_use]
    pub fn at_zero_second(self) -> Self {
        Self { second: Component::exactly_zero(), ..self }
    }

    #[must_use]
    pub fn at_every_second(self) -> Self {
        Self { second: Component::every(), ..self }
    }

    #[must_use]
    pub fn at_every_nth_second(self, n: NonZeroU8) -> Self {
        Self { second: Component::every_step(n), ..self }
    }

    pub fn at_every_second_between(self, range: RangeInclusive<u8>) -> Result<Self, Error> {
        Ok(Self {
            second: Component::between(*range.start(), *range.end()).map_err(|_| Error)?,
            ..self
        })
    }

    pub fn at_every_nth_second_between(
        self,
        range: RangeInclusive<u8>,
        n: NonZeroU8
    ) -> Result<Self, Error>
    {
        Ok(Self {
            second: Component::new(*range.start(), *range.end(), n).map_err(|_| Error)?,
            ..self
        })
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
        // requirements.
        let time = self.advance_to_hms(time);

        match u8::try_from(time.day0()).ok()
            .and_then(|current_d| self.day
                .min_value_bounded(current_d)
                .map(|target_d| (current_d, target_d)))
        {
            Some((current_d, target_d)) if current_d == target_d => time,

            // If the current day is less than the target day and the target day does not exceed
            // the number of days in the current month, then simply set the day to the target day
            // without changing anything else.
            Some((_, target_d)) if target_d < MIN_DAYS
                || i64::from(target_d) < days_in_month(time.year(), time.month()) =>
            {
                time.date()
                    .with_day0(u32::from(target_d))
                    .expect("invalid time")
                    .and_hms(
                        u32::from(self.hour.min_value()),
                        u32::from(self.minute.min_value()),
                        u32::from(self.second.min_value()))
            },

            _ => {
                let target_d = self.day.min_value();
                let (mut year, mut month) = (time.year(), time.month());

                loop {
                    // Advance to the next month
                    (year, month) = match month {
                        12 => (year + 1, 1),
                        _ => (year, month + 1),
                    };

                    if target_d < MIN_DAYS || i64::from(target_d) < days_in_month(year, month) {
                        return time
                            .timezone()
                            .ymd(year, month, u32::from(target_d) + 1)
                            .and_hms(
                                u32::from(self.hour.min_value()),
                                u32::from(self.minute.min_value()),
                                u32::from(self.second.min_value()))
                    }
                }
            },
        }
    }

    fn advance_to_hms<Tz>(self, time: DateTime<Tz>) -> DateTime<Tz>
    where
        Tz: TimeZoneExt,
        Tz::Offset: Copy,
    {
        let time = self.advance_to_ms(time);

        match u8::try_from(time.hour()).ok()
            .and_then(|current_h| self.hour
                .min_value_bounded(current_h)
                .map(|target_h| (current_h, target_h)))
        {
            Some((current_h, target_h)) if current_h == target_h => time,
            Some((_, target_h)) => {
                time.with_hour(u32::from(target_h))
                    .and_then(|time| time.with_minute(u32::from(self.minute.min_value())))
                    .and_then(|time| time.with_second(u32::from(self.second.min_value())))
                    .and_then(|time| time.with_nanosecond(0))
                    .expect("invalid time")
            },
            None => {
                time.date().succ().and_hms(
                    u32::from(self.hour.min_value()),
                    u32::from(self.minute.min_value()),
                    u32::from(self.second.min_value()))
            },
        }
    }

    fn advance_to_ms<Tz>(self, time: DateTime<Tz>) -> DateTime<Tz>
    where
        Tz: TimeZoneExt,
        Tz::Offset: Copy,
    {
        let time = self.advance_to_s(time);

        match u8::try_from(time.minute()).ok()
            .and_then(|current_m| self.minute
                .min_value_bounded(current_m)
                .map(|target_m| (current_m, target_m)))
        {
            Some((current_m, target_m)) if current_m == target_m => time,
            Some((_, target_m)) => {
                time.with_minute(u32::from(target_m))
                    .and_then(|time| time.with_second(u32::from(self.second.min_value())))
                    .and_then(|time| time.with_nanosecond(0))
                    .expect("invalid time")
            },
            None => {
                (time + Duration::hours(1))
                    .with_minute(u32::from(self.minute.min_value()))
                    .and_then(|time| time.with_second(u32::from(self.second.min_value())))
                    .and_then(|time| time.with_nanosecond(0))
                    .expect("invalid time")
            },
        }
    }

    fn advance_to_s<Tz>(self, time: DateTime<Tz>) -> DateTime<Tz>
    where
        Tz: TimeZoneExt,
        Tz::Offset: Copy,
    {
        match u8::try_from(time.second()).ok()
            .and_then(|current_s| self.second
                .min_value_bounded(current_s)
                .map(|target_s| (current_s, target_s)))
        {
            Some((current_s, target_s)) if current_s == target_s => time,
            Some((_, target_s)) => {
                time.with_second(u32::from(target_s))
                    .and_then(|time| time.with_nanosecond(0))
                    .expect("invalid time")
            },
            None => {
                (time + Duration::minutes(1))
                    .with_second(u32::from(self.second.min_value()))
                    .and_then(|time| time.with_nanosecond(0))
                    .expect("invalid time")
            },
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

#[derive(Debug)]
pub struct Error;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid schedule time")
    }
}

impl error::Error for Error {}

#[cfg(test)]
#[allow(clippy::pedantic)]
mod tests {
    use super::Schedule;
    use chrono::{TimeZone, Utc, Timelike};
    use std::num::NonZeroU8;

    #[test]
    fn test_next_occurrence() {
        assert_eq!(
            Schedule::new_every_second()
                .at_hour(11)
                .at_minute(16)
                .at_second(41)
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)),
            Utc.ymd(2022, 4, 5).and_hms(11, 16, 41)
        );

        assert_eq!(
            Schedule::new_every_second()
                .at_hour(18)
                .at_minute(16)
                .at_second(41)
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)),
            Utc.ymd(2022, 4, 4).and_hms(18, 16, 41)
        );

        assert_eq!(
            Schedule::new_every_second()
                .at_hour(11)
                .at_minute(16)
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)),
            Utc.ymd(2022, 4, 5).and_hms(11, 16, 0)
        );

        assert_eq!(
            Schedule::new_every_second()
                .at_hour(11)
                .at_second(14)
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)),
            Utc.ymd(2022, 4, 5).and_hms(11, 0, 14)
        );

        assert_eq!(
            Schedule::new_every_second()
                .at_hour(11)
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)),
            Utc.ymd(2022, 4, 5).and_hms(11, 0, 0)
        );

        assert_eq!(
            Schedule::new_every_second()
                .at_hour(18)
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)),
            Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)
        );

        assert_eq!(
            Schedule::new_every_second()
                .at_hour(19)
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)),
            Utc.ymd(2022, 4, 4).and_hms(19, 0, 0)
        );

        assert_eq!(
            Schedule::new_every_second()
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)),
            Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)
        );

        assert_eq!(
            Schedule::new_every_hour()
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)),
            Utc.ymd(2022, 4, 4).and_hms(19, 0, 0)
        );

        assert_eq!(
            Schedule::new_every_second()
                .at_hour(11)
                .at_minute(16)
                .at_second(41)
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(11, 16, 40)),
            Utc.ymd(2022, 4, 4).and_hms(11, 16, 41)
        );

        assert_eq!(
            Schedule::new_every_second()
                .at_hour(11)
                .at_minute(16)
                .at_second(41)
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(11, 16, 41)),
            Utc.ymd(2022, 4, 4).and_hms(11, 16, 41)
        );

        assert_eq!(
            Schedule::new_every_second()
                .at_hour(11)
                .at_minute(16)
                .at_second(41)
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(11, 16, 42)),
            Utc.ymd(2022, 4, 5).and_hms(11, 16, 41)
        );

        assert_eq!(
            Schedule::new_every_second()
                .at_day(NonZeroU8::new(5).unwrap())
                .at_hour(11)
                .at_minute(16)
                .at_second(41)
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)),
            Utc.ymd(2022, 4, 5).and_hms(11, 16, 41)
        );

        assert_eq!(
            Schedule::new_every_second()
                .at_day(NonZeroU8::new(4).unwrap())
                .at_hour(11)
                .at_minute(16)
                .at_second(41)
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)),
            Utc.ymd(2022, 5, 4).and_hms(11, 16, 41)
        );

        assert_eq!(
            Schedule::new_every_second()
                .at_day(NonZeroU8::new(29).unwrap())
                .at_hour(11)
                .at_minute(16)
                .at_second(41)
                .next_occurrence(Utc.ymd(2022, 2, 10).and_hms(18, 1, 14)),
            Utc.ymd(2022, 3, 29).and_hms(11, 16, 41)
        );

        assert_eq!(
            Schedule::new_every_second()
                .at_day(NonZeroU8::new(29).unwrap())
                .at_hour(11)
                .at_minute(16)
                .at_second(41)
                .next_occurrence(Utc.ymd(2024, 2, 10).and_hms(18, 1, 14)),
            Utc.ymd(2024, 2, 29).and_hms(11, 16, 41)
        );

        assert_eq!(
            Schedule::new_every_second()
                .at_day(NonZeroU8::new(31).unwrap())
                .at_hour(11)
                .at_minute(16)
                .at_second(41)
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)),
            Utc.ymd(2022, 5, 31).and_hms(11, 16, 41)
        );

        assert_eq!(
            Schedule::new_every_second()
                .at_day(NonZeroU8::new(31).unwrap())
                .at_hour(11)
                .at_minute(16)
                .at_second(41)
                .next_occurrence(Utc.ymd(2022, 3, 4).and_hms(18, 1, 14)),
            Utc.ymd(2022, 3, 31).and_hms(11, 16, 41)
        );

        assert_eq!(
            Schedule::new_every_second()
                .at_day(NonZeroU8::new(4).unwrap())
                .at_hour(11)
                .at_minute(16)
                .at_second(41)
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(9, 13, 20)),
            Utc.ymd(2022, 4, 4).and_hms(11, 16, 41)
        );

        assert_eq!(
            Schedule::new_every_second()
                .at_day(NonZeroU8::new(1).unwrap())
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)),
            Utc.ymd(2022, 5, 1).and_hms(0, 0, 0)
        );

        assert_eq!(
            Schedule::new_every_second()
                .at_day(NonZeroU8::new(4).unwrap())
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)),
            Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)
        );

        assert_eq!(
            Schedule::new_every_second()
                .at_day(NonZeroU8::new(5).unwrap())
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)),
            Utc.ymd(2022, 4, 5).and_hms(0, 0, 0)
        );

        assert_eq!(
            Schedule::new_every_second()
                .at_day(NonZeroU8::new(1).unwrap())
                .at_minute(30)
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)),
            Utc.ymd(2022, 5, 1).and_hms(0, 30, 0)
        );

        assert_eq!(
            Schedule::new_every_second()
                .at_day(NonZeroU8::new(9).unwrap())
                .next_occurrence(Utc.ymd(2022, 5, 31).and_hms(18, 1, 14)),
            Utc.ymd(2022, 6, 9).and_hms(0, 0, 0)
        );

        assert_eq!(
            Schedule::new_every_minute()
                .at_every_nth_second(NonZeroU8::new(7).unwrap())
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 0)),
            Utc.ymd(2022, 4, 4).and_hms(18, 1, 0)
        );

        assert_eq!(
            Schedule::new_every_minute()
                .at_every_nth_second(NonZeroU8::new(7).unwrap())
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 1)),
            Utc.ymd(2022, 4, 4).and_hms(18, 1, 7)
        );

        assert_eq!(
            Schedule::new_every_minute()
                .at_every_nth_second(NonZeroU8::new(7).unwrap())
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 7)),
            Utc.ymd(2022, 4, 4).and_hms(18, 1, 7)
        );

        assert_eq!(
            Schedule::new_every_minute()
                .at_every_nth_second(NonZeroU8::new(7).unwrap())
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 8)),
            Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)
        );

        assert_eq!(
            Schedule::new_every_minute()
                .at_every_nth_second(NonZeroU8::new(7).unwrap())
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 56)),
            Utc.ymd(2022, 4, 4).and_hms(18, 1, 56)
        );

        assert_eq!(
            Schedule::new_every_minute()
                .at_every_nth_second(NonZeroU8::new(7).unwrap())
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 57)),
            Utc.ymd(2022, 4, 4).and_hms(18, 2, 0)
        );

        assert_eq!(
            Schedule::new_every_minute()
                .at_every_nth_second(NonZeroU8::new(7).unwrap())
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 59, 57)),
            Utc.ymd(2022, 4, 4).and_hms(19, 0, 0)
        );

        assert_eq!(
            Schedule::new_every_day()
                .at_every_nth_hour(NonZeroU8::new(3).unwrap())
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)),
            Utc.ymd(2022, 4, 4).and_hms(21, 0, 0)
        );

        assert_eq!(
            Schedule::new_every_day()
                .at_every_nth_hour(NonZeroU8::new(3).unwrap())
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(17, 1, 14)),
            Utc.ymd(2022, 4, 4).and_hms(18, 0, 0)
        );

        assert_eq!(
            Schedule::new_every_day()
                .at_every_nth_hour(NonZeroU8::new(3).unwrap())
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(21, 0, 1)),
            Utc.ymd(2022, 4, 5).and_hms(0, 0, 0)
        );

        assert_eq!(
            Schedule::new_every_day()
                .at_every_nth_hour_between(1..=20, NonZeroU8::new(3).unwrap())
                .unwrap()
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(17, 1, 14)),
            Utc.ymd(2022, 4, 4).and_hms(19, 0, 0)
        );

        assert_eq!(
            Schedule::new_every_day()
                .at_every_nth_hour_between(1..=20, NonZeroU8::new(3).unwrap())
                .unwrap()
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(19, 0, 1)),
            Utc.ymd(2022, 4, 5).and_hms(1, 0, 0)
        );
    }

    #[test]
    fn test_next_occurrence_nanos() {
        assert_eq!(
            Schedule::new_every_second()
                .at_hour(11)
                .at_minute(16)
                .at_second(41)
                .next_occurrence(Utc.ymd(2022, 4, 4)
                    .and_hms(11, 16, 41)
                    .with_nanosecond(100)
                    .unwrap()),
            Utc.ymd(2022, 4, 4).and_hms(11, 16, 41).with_nanosecond(100).unwrap()
        );

        assert_eq!(
            Schedule::new_every_second()
                .at_hour(11)
                .at_minute(16)
                .at_second(41)
                .next_occurrence(Utc.ymd(2022, 4, 4)
                    .and_hms(18, 1, 14)
                    .with_nanosecond(100)
                    .unwrap()),
            Utc.ymd(2022, 4, 5).and_hms(11, 16, 41)
        );

        // It doesn't matter that the time has technically passed the start of 18:00:00; the
        // nanoseconds should not be counted when determining whether the time matches the
        // schedule. We are just checking that the day, hour, minute and second match.
        assert_eq!(
            Schedule::new_every_day()
                .at_every_nth_hour(NonZeroU8::new(3).unwrap())
                .next_occurrence(Utc.ymd(2022, 4, 4)
                    .and_hms(18, 0, 0)
                    .with_nanosecond(100)
                    .unwrap()),
            Utc.ymd(2022, 4, 4).and_hms(18, 0, 0).with_nanosecond(100).unwrap()
        );
    }
}
