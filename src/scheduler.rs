use std::thread;
use std::time::Duration as StdDuration;

use chrono::{DateTime, Datelike, Duration, Local, Timelike, Utc};

use crate::timezone_ext::TimeZoneExt;
use crate::task::Task;

struct TaskEntry<Id, Tz>
where
    Tz: TimeZoneExt,
    Tz::Offset: Copy,
{
    task: Task<Id>,
    next_time: Option<DateTime<Tz>>,
}

impl<Id, Tz> TaskEntry<Id, Tz>
where
    Tz: TimeZoneExt,
    Tz::Offset: Copy,
{
    fn new(task: Task<Id>) -> Self {
        Self {
            task,
            next_time: None,
        }
    }

    fn update_next_time(&mut self, now: DateTime<Tz>) {
        match self.next_time {
            Some(next_time) if now < next_time => (),
            _ => self.next_time = self.task.schedule().next_occurrence(now),
        }
    }
}

pub struct ManualSleep<Id, Tz>
where
    Tz: TimeZoneExt,
    Tz::Offset: Copy,
{
    tasks: Vec<TaskEntry<Id, Tz>>,
    next_ids_buf: Vec<Id>,
    previous_time: Option<DateTime<Tz>>,
}

impl<Id, Tz> Default for ManualSleep<Id, Tz>
where
    Tz: TimeZoneExt,
    Tz::Offset: Copy,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Id, Tz> ManualSleep<Id, Tz>
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

impl<Id, Tz> ManualSleep<Id, Tz>
where
    Id: Copy + Eq,
    Tz: TimeZoneExt,
    Tz::Offset: Copy,
{
    #[must_use]
    pub fn with(self, task: Task<Id>) -> Self {
        // Little trick to avoid polluting the function signature with "mut".
        let mut this = self;
        this.insert(task);
        this
    }

    pub fn insert(&mut self, task: Task<Id>) -> bool {
        // Remove existing tasks with IDs equal to the new task's ID, then add the new task. This
        // would be more efficient with a hash map, but we use a vec instead because we will be
        // iterating over the tasks much more often than we will be adding new tasks.
        let removed = self.remove(task.id());
        self.tasks.push(TaskEntry::new(task));
        removed
    }

    pub fn remove(&mut self, id: Id) -> bool {
        let old_len = self.tasks.len();
        self.tasks.retain(|entry| entry.task.id() != id);
        self.tasks.len() != old_len
    }

    #[must_use]
    pub fn contains(&self, id: Id) -> bool {
        self.tasks.iter().any(|entry| entry.task.id() == id)
    }
}

impl<Id, Tz> Iterator for ManualSleep<Id, Tz>
where
    Id: Copy,
    Tz: TimeZoneExt,
    Tz::Offset: Copy,
{
    type Item = (Id, Option<DateTime<Tz>>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(id) = self.next_ids_buf.pop() {
            return Some((id, None));
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
            .filter_map(|entry| {
                entry.update_next_time(min_next_time);
                entry.next_time
            })
            .min()?;

        self.previous_time = Some(next_time);

        // Iterator for all of the tasks whose `next_time` value is equal to the soonest
        // `next_time` value we found above. This may be more than one task, because multiple tasks
        // may want to run at the same time!
        let mut next_ids_iter = self
            .tasks
            .iter()
            .filter(|entry| entry.next_time == Some(next_time))
            .map(|entry| entry.task.id());

        // Get the ID of the first task which should be run at `next_time`. We will return this ID
        // now, so the caller can run the task associated with the ID. If there is no such ID
        // (which can only be the case if there are no tasks in the scheduler), return early with
        // `None` because there is nothing to do in this case.
        let next_id = next_ids_iter.next()?;

        // Add the IDs of any further tasks which should be run at `next_time` to the
        // `next_ids_buf`, so that we can immediately return them in future calls to `next`.
        self.next_ids_buf.extend(next_ids_iter);

        Some((next_id, Some(next_time)))
    }
}

/// An iterator over a collection of tasks. Each call to `next` finds the task that should be run
/// next according to its schedule, sleeps until it should be run, then returns its ID.
pub struct Scheduler<Id, Tz>
where
    Tz: TimeZoneExt,
    Tz::Offset: Copy,
{
    inner: ManualSleep<Id, Tz>,
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
    Tz: TimeZoneExt,
    Tz::Offset: Copy,
{
    #[must_use]
    pub fn new() -> Self {
        Self::from_manual_sleep(ManualSleep::new())
    }

    #[must_use]
    pub fn from_manual_sleep(scheduler: ManualSleep<Id, Tz>) -> Self {
        Self { inner: scheduler }
    }
}

impl<Id, Tz> Scheduler<Id, Tz>
where
    Id: Copy + Eq,
    Tz: TimeZoneExt,
    Tz::Offset: Copy,
{
    #[must_use]
    pub fn with(self, task: Task<Id>) -> Self {
        Self::from_manual_sleep(self.inner.with(task))
    }
    
    pub fn insert(&mut self, task: Task<Id>) -> bool {
        self.inner.insert(task)
    }

    pub fn remove(&mut self, id: Id) -> bool {
        self.inner.remove(id)
    }

    #[must_use]
    pub fn contains(&self, id: Id) -> bool {
        self.inner.contains(id)
    }

    #[must_use]
    pub fn as_manual_sleep(&self) -> &ManualSleep<Id, Tz> {
        &self.inner
    }

    #[must_use]
    pub fn into_manual_sleep(self) -> ManualSleep<Id, Tz> {
        self.inner
    }
}

impl<Id, Tz> Iterator for Scheduler<Id, Tz>
where
    Id: Copy,
    Tz: TimeZoneExt,
    Tz::Offset: Copy,
{
    type Item = Id;

    fn next(&mut self) -> Option<Self::Item> {
        let (id, sleep_until) = self.inner.next()?;
        if let Some(sleep_until) = sleep_until {
            let now = Tz::current_datetime();
            let sleep_duration = (sleep_until - now).to_std().unwrap_or(StdDuration::ZERO);
            if sleep_duration > StdDuration::ZERO {
                thread::sleep(sleep_duration);
            }
        }
        Some(id)
    }
}

#[must_use]
pub fn new_utc<Id>() -> Scheduler<Id, Utc> {
    Scheduler::new()
}

#[must_use]
pub fn new_local<Id>() -> Scheduler<Id, Local> {
    Scheduler::new()
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
