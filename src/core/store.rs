use rusqlite::{Connection, OptionalExtension, Result};
use std::collections::HashMap;

use crate::core::messenger;

use super::task::{Status, Task, Type};
use super::worker::Worker;

// Temporary storage location
const STORAGE_LOCATION: &'static str = "/tmp/fizzbuzzstorage.db";

// ========= STORAGE SUBSCRIPTION ==========

pub async fn subscribe() {
    // println!("STORAGE UP & RUNNING...");
    messenger::subscribe("tasks/status", handle_task_status_update).await;
}

pub async fn handle_task_status_update(m: String) {
    let split: Vec<&str> = m.split(":").collect();

    let task: Task = serde_json::from_str(&m).unwrap();

    updateTask(task);
}

// ========= WORKER STORAGE ==========

pub fn getWorkers() -> Result<Vec<Worker>> {
    let mut workers: Vec<Worker> = Vec::new();

    let connection = getConnection()?;

    let mut query = format!("SELECT * FROM workers");

    let mut stmt = connection.prepare(query.as_str())?;

    let results = stmt.query_map([], |r| {
        Ok(Worker {
            id: r.get(0)?,
            status: r.get(1)?,
        })
    })?;

    for t in results {
        if t.is_err() {
            continue;
        }
        workers.push(t.unwrap());
    }

    Ok(workers)
}

pub fn registerWorker(worker: Worker) -> Result<usize> {
    let connection = getConnection()?;

    let query = format!(
        "INSERT INTO workers VALUES ('{}', '{}')",
        worker.id, worker.status
    );

    connection.execute(&query, [])
}

pub fn updateWorker(worker: Worker) -> Result<usize> {
    let connection = getConnection()?;

    let query = format!(
        "
            UPDATE workers
            SET status = '{}'
            WHERE id = '{}' 
        ",
        worker.status, worker.id
    );

    connection.execute(&query, [])
}

pub fn unsubscribeWorker(id: &str) -> Result<usize> {
    let connection = getConnection()?;

    let query = format!(
        "
          DELETE FROM workers WHERE id = '{}';
        ",
        id
    );

    connection.execute(query.as_str(), ())
}

// ========= TASK STORAGE ==========

pub fn extract_task(data: HashMap<String, String>) -> Result<Option<Task>> {
    if !data.contains_key("id") {
        return Ok(None);
    }

    let kind = Type::retrieve(data.get("kind").unwrap()).unwrap();
    let status: Status = match data.get("status").unwrap().as_str() {
        "Pending" => Status::Pending(data.get("execution_time").unwrap().parse::<u64>().unwrap()),
        "Running" => Status::Running(String::from(data.get("worker_id").unwrap())),
        "Completed" => Status::Completed,
        _ => Status::Completed,
    };

    let task: Task = Task {
        id: String::from(data.get("id").unwrap()),
        kind,
        status,
    };

    Ok(Some(task))
}

pub fn getTask(id: String) -> Result<Option<Task>> {
    let connection = getConnection()?;

    let query = format!("SELECT * FROM tasks WHERE id = '{}'", id);

    let mut task_data: HashMap<String, String> = HashMap::new();

    let mut stmt = connection.prepare(query.as_str())?;
    stmt.query_row([], |r| {
        let execution_time: i64 = r.get_unwrap(3);

        task_data.insert(String::from("id"), r.get(0)?);
        task_data.insert(String::from("kind"), r.get(1)?);
        task_data.insert(String::from("status"), r.get(2)?);
        task_data.insert(
            String::from("execution_time"),
            format!("{}", execution_time),
        );
        task_data.insert(String::from("worker_id"), r.get(4).unwrap_or(String::new()));
        Ok(())
    })
    .optional()?;

    extract_task(task_data)
}

pub fn getTasks(status: Option<&str>) -> Result<Vec<Task>> {
    let mut tasks: Vec<Task> = Vec::new();
    let connection = getConnection()?;

    let mut query = format!("SELECT * FROM tasks");

    if status.is_some() {
        query = format!("{} WHERE status = '{}'", query, status.unwrap());
    }

    let mut stmt = connection.prepare(query.as_str())?;

    let results = stmt.query_map([], |r| {
        let mut task_data: HashMap<String, String> = HashMap::new();

        let execution_time: i64 = r.get_unwrap(3);

        task_data.insert(String::from("id"), r.get(0)?);
        task_data.insert(String::from("kind"), r.get(1)?);
        task_data.insert(String::from("status"), r.get(2)?);
        task_data.insert(
            String::from("execution_time"),
            format!("{}", execution_time),
        );
        task_data.insert(String::from("worker_id"), r.get(4).unwrap_or(String::new()));

        Ok(extract_task(task_data.clone())?.unwrap())
    })?;

    for t in results {
        if t.is_err() {
            continue;
        }
        tasks.push(t.unwrap());
    }

    Ok(tasks)
}

pub fn updateTask(task: Task) -> Result<usize> {
    let connection = getConnection()?;

    let query = format!(
        "
          UPDATE tasks
          SET status = '{}', execution_time = {}, worker_id = '{}'
          WHERE id = '{}'
        ",
        task.status.as_string(),
        task.status.get_execution_time(),
        task.status.get_worker(),
        task.id,
    );

    connection.execute(query.as_str(), ())
}

pub fn deleteTask(id: String) -> Result<usize> {
    let connection = getConnection()?;

    let query = format!(
        "
          DELETE FROM tasks WHERE id = '{}';
        ",
        id
    );

    connection.execute(query.as_str(), ())
}

pub fn storeTask(task: Task) -> Result<usize> {
    let connection = getConnection()?;

    let query = format!(
        "INSERT INTO tasks VALUES ('{}', '{}', '{}', {}, NULL);",
        task.id,
        task.kind.as_string(),
        task.status.as_string(),
        task.status.get_execution_time(),
    );

    connection.execute(query.as_str(), ())
}

pub fn instantiateStorage() -> Result<usize> {
    let connection = getConnection()?;

    let query = "CREATE TABLE IF NOT EXISTS tasks (id TEXT UNIQUE, kind TEXT, status TEXT, execution_time INTEGER, worker_id TEXT);";
    connection.execute(query, ());

    let query: &str = "DROP TABLE IF EXISTS workers";
    connection.execute(query, []);

    let query: &str = "CREATE TABLE IF NOT EXISTS workers (id TEXT UNIQUE, status INTEGER);";
    connection.execute(query, [])
}

fn getConnection() -> Result<Connection> {
    Connection::open(STORAGE_LOCATION)
}

#[cfg(test)]
mod tests {

    use crate::core::{
        store::instantiateStorage,
        task::{Status, Task, Type},
        worker::Worker,
    };

    use super::{deleteTask, getTask, getTasks, registerWorker, storeTask, updateTask};

    #[test]
    fn test_store_get_task() {
        instantiateStorage().unwrap();
        let now = chrono::Utc::now().timestamp() as u64;
        let task = Task::new(now + 10000, String::from("Fizz")).unwrap();
        let result = storeTask(task.clone()).unwrap();

        let queried_task = getTask(task.id.clone()).unwrap();

        assert!(queried_task.is_some());
        assert_eq!(task.id, queried_task.unwrap().id)
    }

    #[test]
    fn test_delete_task() {
        instantiateStorage().unwrap();
        let now = chrono::Utc::now().timestamp() as u64;
        let task = Task::new(now + 10000, String::from("Fizz")).unwrap();
        storeTask(task.clone()).unwrap();

        let delete_task = deleteTask(task.id);

        assert!(delete_task.is_ok());
        assert_eq!(delete_task.unwrap(), 1);
    }

    #[test]
    fn test_update_task() {
        instantiateStorage().unwrap();
        let now = chrono::Utc::now().timestamp() as u64;
        let mut task = Task::new(now + 10000, String::from("FizzBuzz")).unwrap();
        storeTask(task.clone()).unwrap();

        let new_status = Status::Running(String::from("abcdefg"));
        task.updateStatus(new_status.clone());
        let update_task = updateTask(task.clone());

        assert!(update_task.is_ok());

        let queried_task = getTask(task.id).unwrap();

        assert_eq!(
            queried_task.unwrap().status.as_string(),
            new_status.as_string()
        );
    }

    #[test]
    fn test_complete_task() {
        instantiateStorage().unwrap();
        let now = chrono::Utc::now().timestamp() as u64;
        let mut task = Task::new(now + 10000, String::from("FizzBuzz")).unwrap();
        storeTask(task.clone()).unwrap();

        let new_status = Status::Completed;
        task.updateStatus(new_status.clone());
        let update_task = updateTask(task.clone());

        assert!(update_task.is_ok());

        let queried_task = getTask(task.id).unwrap();

        assert_eq!(
            queried_task.unwrap().status.as_string(),
            new_status.as_string()
        );
    }

    #[test]
    fn test_fetch_tasks() {
        let results = getTasks(Some("Pending"));
        assert!(results.is_ok());
        assert!(!results.unwrap().is_empty())
    }

    #[test]
    fn test_register_worker() {
        let x = registerWorker(Worker {
            id: String::from("1234"),
            status: String::from("Idle"),
        })
        .unwrap();
        assert!(x > 0);
    }
}
