use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{thread, time};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Type {
    Fizz,
    Buzz,
    FizzBuzz,
}

impl Type {
    pub fn retrieve(kind: &str) -> Option<Type> {
        match kind {
            "Fizz" => Some(Type::Fizz),
            "Buzz" => Some(Type::Buzz),
            "FizzBuzz" => Some(Type::FizzBuzz),
            _ => None,
        }
    }

    pub fn execute(self, id: &str) -> () {
        thread::sleep(time::Duration::from_secs(self.duration()));
        println!("{} {}", self.as_string(), id)
        // println!("Executing -> {} {}", self.as_string(), id);
    }

    pub fn as_string(self) -> String {
        match self {
            Self::Fizz => String::from("Fizz"),
            Self::Buzz => String::from("Buzz"),
            Self::FizzBuzz => String::from("FizzBuzz"),
        }
    }

    pub fn duration(self) -> u64 {
        match self {
            Self::Fizz => 3,
            Self::Buzz => 5,
            Self::FizzBuzz => 15,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Status {
    Pending(u64),    // unix timemestamp of execution
    Running(String), // worker id
    Completed,
}

impl Status {
    pub fn as_string(&self) -> String {
        match self {
            Status::Pending(x) => String::from("Pending"),
            Status::Running(x) => String::from("Running"),
            Status::Completed => String::from("Completed"),
        }
    }

    pub fn get_execution_time(&self) -> u64 {
        match self {
            Status::Pending(x) => *x,
            _ => 0,
        }
    }

    pub fn get_worker(&self) -> String {
        match self {
            Status::Running(x) => String::from(x),
            _ => String::from(""),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub kind: Type,
    pub status: Status,
}

impl Task {
    pub fn new(when: u64, kind: String) -> Result<Task, &'static str> {
        let kind = Type::retrieve(kind.as_str());

        if kind.is_none() {
            return Err("Error while creating the new task, type was incorrect.");
        }

        Ok(Task {
            id: Task::randomId(),
            kind: kind.unwrap(),
            status: Status::Pending(when),
        })
    }

    pub fn updateStatus(&mut self, status: Status) -> () {
        self.status = status;
    }

    pub fn execute(&mut self, workerId: &str) -> Result<(), &'static str> {
        match self.status {
            Status::Pending(execution_time) => {
                if Utc::now().timestamp() >= execution_time as i64 {
                    // Return ok even if it's not executing the task, wait for the next
                    return Ok(());
                }

                self.updateStatus(Status::Running(String::from(workerId)));
                self.kind.execute(self.id.as_str());
                self.updateStatus(Status::Completed);

                Ok(())
            }
            _ => Err("Task is not in pending mode. It cannot be executed..."),
        }
    }

    fn randomId() -> String {
        random_string::generate(
            16,
            "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ",
        )
    }
}
