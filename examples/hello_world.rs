use chrono::prelude::*;
use tasque::*;

#[derive(Copy, Clone, PartialEq, Eq)]
enum TaskId {
    Task1,
    Task2,
}

fn main() {
    let mut scheduler = Scheduler::<TaskId, _>::new_utc();
    scheduler.insert(Task::builder(TaskId::Task1).at(Schedule::new_every_minute().at_second(15)).build());
    scheduler.insert(Task::builder(TaskId::Task2).at(Schedule::new_every_minute().at_second(45)).build());

    for id in scheduler.take(4) {
        run_task(id);
    }
}

fn run_task(id: TaskId) {
    print!("{:?} ", Utc::now());
    match id {
        TaskId::Task1 => println!("Hello, world! The current second is now 15"),
        TaskId::Task2 => println!("Hello, world! The current second is now 45"),
    }
}
