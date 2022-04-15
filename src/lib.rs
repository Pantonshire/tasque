#![warn(clippy::pedantic)]

mod schedules;
pub mod schedule;
pub mod scheduler;

use std::marker::PhantomData;

use chrono::{DateTime, TimeZone};
pub use chrono::{self, Local, Utc};

use schedules::Schedules;
pub use schedule::Schedule;
pub use scheduler::{Scheduler, ManualSleep as ManualSleepScheduler};

// impl<Id> Scheduler<Id, Utc> {
//     #[must_use]
//     pub fn new_utc() -> Self {
//         Self::new()
//     }
// }

// impl<Id> Scheduler<Id, Local> {
//     #[must_use]
//     pub fn new_local() -> Self {
//         Self::new()
//     }
// }

pub struct TaskBuilder<Id, Tz>
where
    Tz: TimeZoneExt,
    Tz::Offset: Copy,
{
    identifier: Id,
    times: Vec<Schedule>,
    _tz_marker: PhantomData<Tz>,
}

impl<Id, Tz> TaskBuilder<Id, Tz>
where
    Tz: TimeZoneExt,
    Tz::Offset: Copy,
{
    #[must_use]
    pub fn new(id: Id) -> Self {
        Self {
            identifier: id,
            times: Vec::new(),
            _tz_marker: PhantomData,
        }
    }

    #[must_use]
    pub fn build(self) -> Task<Id, Tz> {
        Task::new(self.identifier, Schedules::from_vec(self.times))
    }

    #[must_use]
    pub fn at(mut self, time: Schedule) -> Self {
        self.times.push(time);
        self
    }

    #[must_use]
    pub fn at_several<T>(mut self, times: T) -> Self
    where
        T: IntoIterator<Item = Schedule>,
    {
        self.times.extend(times);
        self
    }
}

pub struct Task<Id, Tz>
where
    Tz: TimeZoneExt,
    Tz::Offset: Copy,
{
    identifier: Id,
    schedule: Schedules,
    next_time: Option<DateTime<Tz>>,
}

impl<Id, Tz> Task<Id, Tz>
where
    Tz: TimeZoneExt,
    Tz::Offset: Copy,
{
    #[must_use]
    pub fn builder(id: Id) -> TaskBuilder<Id, Tz> {
        TaskBuilder::new(id)
    }

    fn new(id: Id, schedule: Schedules) -> Self {
        Self {
            identifier: id,
            schedule,
            next_time: None,
        }
    }

    #[inline]
    fn update_next_time(&mut self, now: DateTime<Tz>) {
        match self.next_time {
            Some(next_time) if now < next_time => (),
            _ => self.next_time = self.schedule.next_occurrence(now),
        }
    }
}

pub trait TimeZoneExt
where
    Self: TimeZone,
    Self::Offset: Copy,
{
    fn current_datetime() -> DateTime<Self>;
}

impl TimeZoneExt for Utc {
    #[inline]
    fn current_datetime() -> DateTime<Self> {
        Utc::now()
    }
}

impl TimeZoneExt for Local {
    #[inline]
    fn current_datetime() -> DateTime<Self> {
        Local::now()
    }
}
