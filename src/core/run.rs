use crate::core;
use std::thread;

use super::utils::RunType;

pub async fn master(address: String) {
    core::store::instantiateStorage().unwrap();

    run_webserver(address);
    run_worker();

    // STORE LISTENER
    thread::spawn(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                core::store::subscribe().await;
            });
    });

    // SCHEDULER LISTENER
    thread::spawn(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                core::scheduler::run().await;
            });
    });

    main_loop(RunType::Master).await;
}

pub async fn slave(address: String) {
    core::store::instantiateStorage().unwrap();

    run_webserver(address);
    run_worker();

    main_loop(RunType::Slave).await;
}

async fn main_loop(runType: RunType) {
    loop {
        thread::sleep(std::time::Duration::from_secs(1));
        core::worker::register().await;
        match runType {
            RunType::Master => core::scheduler::launch_jobs().await,
            _ => ()
        }
    }
}

fn run_webserver(address: String) {
    // WEB API
    thread::spawn(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                // TODO: handle not bindable address
                core::web::run(address).await;
            });
    });
}

fn run_worker() {
    // WORKER
    thread::spawn(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                core::worker::run().await;
            });
    });
}
