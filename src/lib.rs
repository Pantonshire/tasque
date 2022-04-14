#![warn(clippy::pedantic)]

pub mod schedule;
mod schedules;

use std::marker::PhantomData;
use std::ops::{Range, RangeInclusive};
use std::thread;
use std::time::Duration as StdDuration;

use chrono::{DateTime, Datelike, Duration, Local, TimeZone, Timelike, Utc};

use schedules::Schedules;

pub use schedule::Schedule;

pub struct Scheduler<Id, Tz>
where
    Tz: TimeZoneExt,
    Tz::Offset: Copy,
{
    tasks: Vec<Task<Id, Tz>>,
    next_ids_buf: Vec<Id>,
    previous_time: Option<DateTime<Tz>>,
}

impl<Id, Tz> Scheduler<Id, Tz>
where
    Tz: TimeZoneExt,
    Tz::Offset: Copy,
{
    #[must_use]
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            next_ids_buf: Vec::new(),
            previous_time: None,
        }
    }
}

impl<Id, Tz> Default for Scheduler<Id, Tz>
where
    Tz: TimeZoneExt,
    Tz::Offset: Copy,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Id, Tz> Scheduler<Id, Tz>
where
    Id: Copy + Eq,
    Tz: TimeZoneExt,
    Tz::Offset: Copy,
{
    pub fn insert(&mut self, task: Task<Id, Tz>) {
        // Remove existing tasks with IDs equal to the new task's ID, then add the new task. This
        // would be more efficient with a hash map, but we use a vec instead because we will be
        // iterating over the tasks much more often than we will be adding new tasks.
        self.remove(task.identifier);
        self.tasks.push(task);
    }

    pub fn remove(&mut self, id: Id) {
        self.tasks
            .retain(|existing_task| existing_task.identifier != id);
    }
}

impl<Id, Tz> Iterator for Scheduler<Id, Tz>
where
    Id: Copy,
    Tz: TimeZoneExt,
    Tz::Offset: Copy,
{
    type Item = Id;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(id) = self.next_ids_buf.pop() {
            return Some(id);
        }

        // The soonest time at which we will run the next task.
        let min_next_time = {
            let now = Tz::current_datetime();

            // Use the cached previous step time as a guard against non-monotonic time and
            // inaccurate sleeping times. The previous iteration was supposed to sleep until
            // `previous_step`, so if `previous_step` is greater than the reported current time,
            // consider `previous_time` to be the current time instead.
            let now = match self.previous_time {
                Some(previous_time) => now.max(previous_time),
                None => now,
            };

            // We will run tasks no sooner than the start of the next second after the current one.
            // This prevents tasks from being run multiple times per second if `next` is called
            // multiple times per second.
            next_second(now)
        };

        // Re-calculate the `next_time` values for all tasks using `min_next_time` as a lower
        // bound, then find which one is soonest.
        let next_time = self
            .tasks
            .iter_mut()
            .filter_map(|task| {
                task.update_next_time(min_next_time);
                task.next_time
            })
            .min()?;

        self.previous_time = Some(next_time);

        // Iterator for all of the tasks whose `next_time` value is equal to the soonest
        // `next_time` value we found above. This may be more than one task, because multiple tasks
        // may want to run at the same time!
        let mut next_ids_iter = self
            .tasks
            .iter()
            .filter(|task| task.next_time == Some(next_time))
            .map(|task| task.identifier);

        // Get the ID of the first task which should be run at `next_time`. We will return this ID
        // now, so the caller can run the task associated with the ID. If there is no such ID
        // (which can only be the case if there are no tasks in the scheduler), return early with
        // `None` because there is nothing to do in this case.
        let next_id = next_ids_iter.next()?;

        // Add the IDs of any further tasks which should be run at `next_time` to the
        // `next_ids_buf`, so that we can immediately return them in future calls to `next`.
        self.next_ids_buf.extend(next_ids_iter);

        let now = Tz::current_datetime();
        let sleep_duration = (next_time - now).to_std().unwrap_or(StdDuration::ZERO);
        if sleep_duration > StdDuration::ZERO {
            thread::sleep(sleep_duration);
        }

        Some(next_id)
    }
}

#[inline]
fn next_second<Tz>(time: DateTime<Tz>) -> DateTime<Tz>
where
    Tz: TimeZoneExt,
    Tz::Offset: Copy,
{
    let time = time + Duration::seconds(1);
    time.timezone()
        .ymd(time.year(), time.month(), time.day())
        .and_hms(time.hour(), time.minute(), time.second())
}

impl<Id> Scheduler<Id, Utc> {
    #[must_use]
    pub fn new_utc() -> Self {
        Self::new()
    }
}

impl<Id> Scheduler<Id, Local> {
    #[must_use]
    pub fn new_local() -> Self {
        Self::new()
    }
}

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

pub trait IntoInclusiveRange<T: Ord> {
    /// If `Some((start, end))` is returned, it should always satisfy `start <= end`. In cases
    /// where this condition cannot be satisfied, `None` should be returned instead.
    fn into_inclusive_range(self) -> Option<(T, T)>;
}

impl IntoInclusiveRange<u8> for RangeInclusive<u8> {
    fn into_inclusive_range(self) -> Option<(u8, u8)> {
        let range @ (start, end) = (*self.start(), *self.end());
        if start <= end {
            Some(range)
        } else {
            None
        }
    }
}

impl IntoInclusiveRange<u8> for Range<u8> {
    fn into_inclusive_range(self) -> Option<(u8, u8)> {
        if self.start < self.end {
            Some((self.start, self.end - 1))
        } else {
            None
        }
    }
}
