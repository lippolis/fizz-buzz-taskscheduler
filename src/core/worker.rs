use crate::core::task::Task;
use std::thread;

use super::{messenger, task::Status};
use std::env;

pub struct Worker {
    pub id: String,
    pub status: String,
}

pub async fn register() {
    let id = env::var(format!("{}_FB_WORKER_ID", std::process::id()).as_str());

    if id.is_err() {
        return;
    };

    // Register worker to the network...
    messenger::send_message(
        "worker/index",
        format!("{}:register", &id.unwrap()).as_str(),
    )
    .await;
}

pub async fn run() {
    let id = randomId();
    // let id = String::from("at4M2eqcJEWzpl9I");
    env::set_var(format!("{}_FB_WORKER_ID", std::process::id()).as_str(), &id);
    // Handle messages
    messenger::subscribe(format!("worker/jobs/{}", id).as_str(), handle_message).await;
}

async fn handle_message(message: String) -> () {
    let id = env::var(format!("{}_FB_WORKER_ID", std::process::id()).as_str()).unwrap();
    messenger::send_message("worker/index", format!("{}:working", &id).as_str()).await;
    println!("[WORKER-{}] Received task: {}", &id, message);

    let mut task: Task = serde_json::from_str(&message).unwrap();

    task.kind.execute(&task.id);

    // Completed Task
    task.updateStatus(Status::Completed);
    let serialized_task = serde_json::to_string(&task).unwrap();
    messenger::send_message("tasks/status", serialized_task.as_str()).await;

    messenger::send_message("worker/index", format!("{}:idle", &id).as_str()).await;
    ()
}

fn randomId() -> String {
    random_string::generate(
        16,
        "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ",
    )
}
