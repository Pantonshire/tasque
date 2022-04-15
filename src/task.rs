use crate::schedule::Schedule;
use crate::schedules::Schedules;

pub struct Builder<Id> {
    id: Id,
    times: Vec<Schedule>,
}

impl<Id> Builder<Id> {
    #[must_use]
    pub fn new(id: Id) -> Self {
        Self {
            id,
            times: Vec::new(),
        }
    }

    #[must_use]
    pub fn build(self) -> Task<Id> {
        Task::new(self.id, Schedules::from_vec(self.times))
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

pub struct Task<Id> {
    id: Id,
    schedule: Schedules,
}

impl<Id> Task<Id> {
    #[must_use]
    pub fn builder(id: Id) -> Builder<Id> {
        Builder::new(id)
    }

    #[must_use]
    fn new(id: Id, schedule: Schedules) -> Self {
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
