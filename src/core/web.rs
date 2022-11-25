use std::collections::HashMap;

use graphul::{extract::Json, http::Methods, Context, Graphul};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{
    messenger,
    store::{deleteTask, getTask, getTasks, storeTask},
    task::Task,
};

#[derive(Deserialize, Serialize, Default, Debug)]
struct SetBodyRequest {
    kind: String,
    when: u64,
}

pub async fn run(address: String) -> () {
    let mut app = Graphul::new();

    // Set Task
    app.post("/task/set", |c: Context| async move {
        let value: Json<SetBodyRequest> = match c.payload().await {
            Ok(data) => data,
            Err(err) => {
                return Json(
                    json!({ "id": "", "success": false, "message": "Error on parameters" }),
                )
            }
        };

        let kind = value.0.kind;
        let when = value.0.when;

        // NOTE: storage can be distributed on nodes...
        // messenger::send_message("tasks/push", format!("{:?}:{}", kind, when).as_str()).await;

        let task = Task::new(when, kind).unwrap();
        let write = storeTask(task.clone());

        Json(json!({ "id": task.id, "success": write.is_ok(), "message": "Task created" }))
    });

    // Delete Task
    app.post("/task/delete/:id", |c: Context| async move {
        // TODO: deletion is not correct, you have to check if task is running. You
        // can delete only pending and completed...
        let id = c.params("id");
        Json(json!({ "id": id, "success": deleteTask(id).is_ok(), "message": "Perform deletion" }))
    });

    // List Task
    app.get("/task", |c: Context| async move {
        // TODO: add filtering by type and sort by time!
        let mut status: Option<&str> = None;
        let status_query = c.query("status");
        if !status_query.is_empty() {
            status = Some(&status_query);
        }

        match getTasks(status) {
            Result::Ok(tasks) => {
                return Json(json!({ "success": true, "tasks": tasks, "status": status }))
            }
            _ => return Json(json!({ "success": false, "tasks": [], "status": status })),
        }
    });

    // Get Task
    app.get("/task/:id", |c: Context| async move {
        let id = c.params("id");

        match getTask(id) {
            Result::Ok(t) => match t {
                Option::Some(t) => return Json(json!({ "success": true, "task": t })),
                _ => return Json(json!({ "success": false })),
            },
            _ => return Json(json!({ "success": false })),
        }
    });

    app.run(address.as_str()).await;
}
