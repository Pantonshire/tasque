use std::slice;

use chrono::DateTime;

use crate::task_time::TaskTime;
use crate::TimeZoneExt;

pub(crate) struct TaskTimeBuf {
    repr: TaskTimeBufRepr,
}

impl TaskTimeBuf {
    pub fn new_single(time: TaskTime) -> Self {
        Self {
            repr: TaskTimeBufRepr::new_from_array([time]),
        }
    }

    pub fn new_from_vec(vec: Vec<TaskTime>) -> Self {
        Self {
            repr: TaskTimeBufRepr::new_from_vec(vec),
        }
    }

    pub fn new_from_array<const N: usize>(array: [TaskTime; N]) -> Self {
        Self {
            repr: TaskTimeBufRepr::new_from_array(array),
        }
    }

    #[inline]
    pub fn iter(&self) -> slice::Iter<TaskTime> {
        self.repr.iter()
    }

    #[inline]
    pub fn next_occurrence<Tz>(&self, now: DateTime<Tz>) -> Option<DateTime<Tz>>
    where
        Tz: TimeZoneExt,
        Tz::Offset: Copy,
    {
        self.iter().map(|time| time.next_occurrence(now)).min()
    }
}

enum TaskTimeBufRepr {
    Stack {
        buf: [TaskTime; Self::ARRAY_LEN],
        len: u8,
    },
    Heap {
        buf: Box<[TaskTime]>,
    },
}

impl TaskTimeBufRepr {
    const ARRAY_LEN: usize = 5;

    fn new_from_vec(vec: Vec<TaskTime>) -> Self {
        let n = vec.len();
        if n <= Self::ARRAY_LEN {
            let mut buf = [TaskTime::default(); Self::ARRAY_LEN];
            buf[..n].copy_from_slice(&vec);
            Self::Stack { buf, len: u8::try_from(n).unwrap() }
        } else {
            Self::Heap {
                buf: vec.into_boxed_slice(),
            }
        }
    }

    fn new_from_array<const N: usize>(array: [TaskTime; N]) -> Self {
        if N <= Self::ARRAY_LEN {
            let mut buf = [TaskTime::default(); Self::ARRAY_LEN];
            buf[..N].copy_from_slice(&array);
            Self::Stack { buf, len: u8::try_from(N).unwrap() }
        } else {
            Self::Heap { buf: array.into() }
        }
    }

    #[inline]
    fn iter(&self) -> slice::Iter<TaskTime> {
        match self {
            TaskTimeBufRepr::Stack { buf, len } => buf[..*len as usize].iter(),
            TaskTimeBufRepr::Heap { buf } => buf.iter(),
        }
    }
}

impl<'a> IntoIterator for &'a TaskTimeBufRepr {
    type Item = &'a TaskTime;

    type IntoIter = slice::Iter<'a, TaskTime>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
#[allow(clippy::pedantic)]
mod tests {
    use std::num::NonZeroU8;

    use chrono::{TimeZone, Utc};

    use super::TaskTimeBuf;
    use crate::task_time::TaskTime;

    const TT1: TaskTime = TaskTime::builder()
        .at_hour(10)
        .at_minute(30)
        .at_second(0)
        .build();

    const TT2: TaskTime = TaskTime::builder()
        .at_day(unsafe { NonZeroU8::new_unchecked(9) })
        .build();

    const TT3: TaskTime = TaskTime::builder().at_minute(5).at_second(0).build();

    const TT4: TaskTime = TaskTime::builder().at_second(15).build();

    const TT5: TaskTime = TaskTime::builder()
        .at_day(unsafe { NonZeroU8::new_unchecked(11) })
        .at_hour(10)
        .at_minute(15)
        .at_second(0)
        .build();

    const TT6: TaskTime = TaskTime::builder().at_hour(12).at_second(40).build();

    const TT7: TaskTime = TaskTime::builder()
        .at_day(unsafe { NonZeroU8::new_unchecked(31) })
        .at_second(0)
        .build();

    #[test]
    fn test_task_time_buf_iter() {
        let buf = TaskTimeBuf::new_from_vec(vec![]);
        let mut iter = buf.iter();
        assert_eq!(iter.next(), None);

        let buf = TaskTimeBuf::new_from_vec(vec![TT1]);
        let mut iter = buf.iter();
        assert_eq!(iter.next(), Some(&TT1));
        assert_eq!(iter.next(), None);

        let buf = TaskTimeBuf::new_from_vec(vec![TT1, TT2, TT3, TT4, TT5]);
        let mut iter = buf.iter();
        assert_eq!(iter.next(), Some(&TT1));
        assert_eq!(iter.next(), Some(&TT2));
        assert_eq!(iter.next(), Some(&TT3));
        assert_eq!(iter.next(), Some(&TT4));
        assert_eq!(iter.next(), Some(&TT5));
        assert_eq!(iter.next(), None);

        let buf = TaskTimeBuf::new_from_vec(vec![TT1, TT2, TT3, TT4, TT5, TT6]);
        let mut iter = buf.iter();
        assert_eq!(iter.next(), Some(&TT1));
        assert_eq!(iter.next(), Some(&TT2));
        assert_eq!(iter.next(), Some(&TT3));
        assert_eq!(iter.next(), Some(&TT4));
        assert_eq!(iter.next(), Some(&TT5));
        assert_eq!(iter.next(), Some(&TT6));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_task_time_buf_next_occurrence() {
        assert_eq!(
            TaskTimeBuf::new_from_vec(vec![])
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)),
            None
        );

        assert_eq!(
            TaskTimeBuf::new_from_vec(vec![TT1])
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)),
            Some(Utc.ymd(2022, 4, 5).and_hms(10, 30, 0))
        );

        assert_eq!(
            TaskTimeBuf::new_from_vec(vec![TT1, TT2, TT3, TT4, TT5])
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)),
            Some(Utc.ymd(2022, 4, 4).and_hms(18, 1, 15))
        );

        assert_eq!(
            TaskTimeBuf::new_from_vec(vec![TT1, TT2, TT3, TT5])
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)),
            Some(Utc.ymd(2022, 4, 4).and_hms(18, 5, 0))
        );

        assert_eq!(
            TaskTimeBuf::new_from_vec(vec![TT1, TT2, TT3, TT4, TT5, TT6, TT7])
                .next_occurrence(Utc.ymd(2022, 4, 4).and_hms(18, 1, 14)),
            Some(Utc.ymd(2022, 4, 4).and_hms(18, 1, 15))
        );

        assert_eq!(
            TaskTimeBuf::new_from_vec(vec![TT1, TT2, TT3, TT4, TT5, TT6, TT7])
                .next_occurrence(Utc.ymd(2022, 5, 31).and_hms(18, 1, 14)),
            Some(Utc.ymd(2022, 5, 31).and_hms(18, 1, 15))
        );

        assert_eq!(
            TaskTimeBuf::new_from_vec(vec![TT1, TT2, TT3, TT5, TT6, TT7])
                .next_occurrence(Utc.ymd(2022, 5, 31).and_hms(18, 1, 14)),
            Some(Utc.ymd(2022, 5, 31).and_hms(18, 2, 0))
        );

        assert_eq!(
            TaskTimeBuf::new_from_vec(vec![TT1, TT2, TT3, TT4, TT5, TT6, TT7])
                .next_occurrence(Utc.ymd(2022, 4, 9).and_hms(18, 1, 14)),
            Some(Utc.ymd(2022, 4, 9).and_hms(18, 1, 14))
        );
    }

    #[test]
    fn test_task_time_buf_small() {
        use std::mem::size_of;
        assert!(size_of::<TaskTimeBuf>() <= size_of::<Vec<TaskTime>>());
    }
}
