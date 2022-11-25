use super::{
    messenger,
    store::{self, getTasks, getWorkers},
    task::{Status, Task},
    worker::Worker,
};

pub async fn launch_jobs() {
    let now = chrono::Utc::now().timestamp() as u64;
    let tasks = getTasks(Some("Pending")).unwrap();
    let workers = getWorkers().unwrap();

    let mut runnable = tasks
        .iter()
        .filter(|x| x.status.get_execution_time() <= now)
        .collect::<Vec<&Task>>();

    if runnable.is_empty() {
        return ();
    }

    let available_workers = workers
        .iter()
        .filter(|x| x.status == "Idle")
        .collect::<Vec<&Worker>>();

    for worker in available_workers {
        if runnable.is_empty() {
            break;
        }
        let mut task = runnable.remove(0).clone();
        let serialized_task = serde_json::to_string(&task).unwrap();
        messenger::send_message(
            format!("worker/jobs/{}", worker.id).as_str(),
            &serialized_task,
        )
        .await;

        // Update task status...
        task.updateStatus(Status::Running(String::from(&worker.id)));
        let serialized_task = serde_json::to_string(&task).unwrap();
        messenger::send_message("tasks/status", serialized_task.as_str()).await;
    }
}

pub async fn run() {
    // println!("SCHEDULER UP & RUNNING... ");
    messenger::subscribe("worker/index", handle_worker_message).await;
}

async fn handle_worker_message(m: String) {
    // println!("SCHEDULER -> {}", m);

    let split: Vec<&str> = m.split(":").collect();

    let worker_id = format!("{}", split.first().unwrap());
    let action = format!("{}", split.last().unwrap());

    // println!("ACTION {}", action);

    if action == "register" {
        let store = store::registerWorker(Worker {
            id: worker_id,
            status: String::from("Idle"),
        });

        if store.is_err() {
            // println!("Worker already registered...");
        }
        return;
    }

    if action == "working" {
        store::updateWorker(Worker {
            id: worker_id,
            status: String::from("Working"),
        })
        .unwrap();
        return;
    }

    if action == "idle" {
        store::updateWorker(Worker {
            id: worker_id,
            status: String::from("Idle"),
        })
        .unwrap();
        return;
    }

    if action == "delete" {
        store::unsubscribeWorker(&worker_id).unwrap();
        return;
    }
}
