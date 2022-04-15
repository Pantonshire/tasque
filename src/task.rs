use crate::schedule::Schedule;
use crate::schedules::Schedules;

pub struct Task<Id> {
    id: Id,
    schedule: Schedules,
}

impl<Id> Task<Id> {
    pub fn new(id: Id, schedule: Schedule) -> Self {
        Self::new_internal(id, Schedules::One(schedule))
    }

    pub fn new_multi_schedule<T: IntoIterator<Item = Schedule>>(id: Id, schedules: T) -> Self {
        Self::new_multi_schedule_vec(id, schedules.into_iter().collect())
    }

    pub fn new_multi_schedule_vec(id: Id, schedules: Vec<Schedule>) -> Self {
        Self::new_internal(id, Schedules::from_vec(schedules))
    }

    #[must_use]
    fn new_internal(id: Id, schedule: Schedules) -> Self {
        Self {
            id,
            schedule,
        }
    }

    #[must_use]
    pub fn id_ref(&self) -> &Id {
        &self.id
    }

    pub(crate) fn schedule(&self) -> &Schedules {
        &self.schedule
    }
}

impl<Id> Task<Id>
where
    Id: Copy,
{
    #[must_use]
    pub fn id(&self) -> Id {
        self.id
    }
}
