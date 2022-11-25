use std::future::Future;
use tokio;
use zenoh::prelude::r#async::*;

pub async fn send_message(path: &str, value: &str) {
    let session = zenoh::open(config::default()).res().await.unwrap();
    session.put(path, value).res().await.unwrap();
    session.close().res().await.unwrap();
}

pub async fn subscribe<F, T>(path: &str, f: F)
where
    F: Fn(String) -> T,
    T: Future<Output = ()> + Send + 'static,
{
    let session = zenoh::open(config::default()).res().await.unwrap();
    let subscriber = session.declare_subscriber(path).res().await.unwrap();
    while let Ok(m) = subscriber.recv_async().await {
        tokio::spawn(f(m.value.to_string()));
    }
}