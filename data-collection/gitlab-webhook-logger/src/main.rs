#![deny(warnings)]
use std::{convert::Infallible, env};

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::{hyper::body::Bytes, Filter};

use std::fs::OpenOptions;
use std::io::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

fn append_entry(event: String, payload: Bytes) -> Result<(), std::io::Error> {
    let path = if let Some(arg1) = env::args().nth(1) {
        arg1
    } else {
        "/tmp/gitlab-webhook-payloads".to_string()
    };

    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(path)
        .unwrap();

    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    write!(
        file,
        "Timestamp:{:?}\nX-Gitlab-Event:{}\n",
        since_the_epoch.as_secs(),
        event
    )?;
    file.write(&payload)?;
    write!(file, "\n--- ---\n")?;

    Ok(())
}

async fn handler(
    context: Arc<Mutex<i64>>,
    event: String,
    payload: Bytes,
) -> Result<impl warp::Reply, Infallible> {
    println!("X-Gitlab-Event: {}", event);

    let lock = (*context).lock().await;
    if let Err(e) = append_entry(event, payload) {
        eprintln!("Failed to write payload to file: {}", e);
    }

    Ok(format!("OK {}", lock))
}

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    if let Some(arg1) = env::args().nth(1) {
        println!("Using {} as payload file", arg1);
    }

    let file_lock = Arc::new(Mutex::new(42));

    let gitlab_header =
        warp::header::exact("X-Gitlab-Token", "7A6FDE5D-DABC-4F7B-A6EE-FFD22D3E4E7F");
    let routes = gitlab_header
        .map(move || return file_lock.clone())
        .and(warp::header("X-Gitlab-Event"))
        .and(warp::body::bytes())
        .and_then(handler);

    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;

    Ok(())
}
