use chrono::DateTime;

use crate::schedule::Schedule;
use crate::TimeZoneExt;

pub(crate) enum Schedules {
    One(Schedule),
    Many(Box<[Schedule]>),
}

impl Schedules {
    pub(crate) fn from_vec(schedules: Vec<Schedule>) -> Self {
        match schedules.len() {
            1 => Self::One(schedules[0]),
            _ => Self::Many(schedules.into_boxed_slice()),
        }
    }

    pub(crate) fn from_array<const N: usize>(schedules: [Schedule; N]) -> Self {
        match N {
            1 => Self::One(schedules[0]),
            _ => Self::Many(schedules.into()),
        }
    }

    #[inline]
    pub(crate) fn next_occurrence<Tz>(&self, now: DateTime<Tz>) -> Option<DateTime<Tz>>
    where
        Tz: TimeZoneExt,
        Tz::Offset: Copy,
    {
        match self {
            Self::One(schedule) => {
                Some(schedule.next_occurrence(now))
            },
            Self::Many(schedules) => {
                schedules.iter()
                    .map(|schedule| schedule.next_occurrence(now))
                    .min()
            },
        }
    }
}

#[cfg(test)]
#[allow(clippy::pedantic)]
mod tests {
    use std::num::NonZeroU8;

    use chrono::{TimeZone, Utc};

    use super::Schedules;
    use crate::schedule::Schedule;

    #[test]
    fn test_task_time_buf_next_occurrence() {
        let tt1 = Schedule::new_every_day()
            .at_hour(10)
            .at_minute(30)
            .at_second(0);

        let tt2 = Schedule::new_every_second()
            .at_day(NonZeroU8::new(9).unwrap());

        let tt3 = Schedule::new_every_second()
            .at_minute(5)
            .at_second(0);

        let tt4 = Schedule::new_every_second()
            .at_second(15);

        let tt5 = Schedule::new_every_second()
            .at_day(NonZeroU8::new(11).unwrap())
            .at_hour(10)
            .at_minute(15)
            .at_second(0);

        let tt6: Schedule = Schedule::new_every_second()
            .at_hour(12)
            .at_second(40);

        let tt7: Schedule = Schedule::new_every_second()
            .at_day(NonZeroU8::new(31).unwrap())
            .at_second(0);

        assert_eq!(
            Schedules::from_vec(vec![])
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)),
            None
        );

        assert_eq!(
            Schedules::from_vec(vec![tt1])
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)),
            Some(Utc.ymd(2022, 4, 5).and_hms(10, 30, 0))
        );

        assert_eq!(
            Schedules::from_vec(vec![tt1, tt2, tt3, tt4, tt5])
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)),
            Some(Utc.ymd(2022, 4, 4).and_hms(18, 1, 15))
        );

        assert_eq!(
            Schedules::from_vec(vec![tt1, tt2, tt3, tt5])
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)),
            Some(Utc.ymd(2022, 4, 4).and_hms(18, 5, 0))
        );

        assert_eq!(
            Schedules::from_vec(vec![tt1, tt2, tt3, tt4, tt5, tt6, tt7])
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)),
            Some(Utc.ymd(2022, 4, 4).and_hms(18, 1, 15))
        );

        assert_eq!(
            Schedules::from_vec(vec![tt1, tt2, tt3, tt4, tt5, tt6, tt7])
                .next_occurrence(Utc.ymd(2022, 5, 31).and_hms(18, 1, 14)),
            Some(Utc.ymd(2022, 5, 31).and_hms(18, 1, 15))
        );

        assert_eq!(
            Schedules::from_vec(vec![tt1, tt2, tt3, tt5, tt6, tt7])
                .next_occurrence(Utc.ymd(2022, 5, 31).and_hms(18, 1, 14)),
            Some(Utc.ymd(2022, 5, 31).and_hms(18, 2, 0))
        );

        assert_eq!(
            Schedules::from_vec(vec![tt1, tt2, tt3, tt4, tt5, tt6, tt7])
                .next_occurrence(Utc.ymd(2022, 4, 9).and_hms(18, 1, 14)),
            Some(Utc.ymd(2022, 4, 9).and_hms(18, 1, 14))
        );
    }

    #[test]
    fn test_task_time_buf_small() {
        use std::mem::size_of;
        assert!(size_of::<Schedules>() <= size_of::<Vec<Schedule>>());
    }
}
