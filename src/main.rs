use std::{
    collections::{hash_map::Entry, HashMap},
    os::unix::net::UnixDatagram,
    path::PathBuf,
};

use anyhow::Result;
use clap::Parser;
use notify_rust::{Hint, Notification};
use serde::Deserialize;

#[derive(Deserialize)]
struct NotificationRequest {
    tag: String,
    body: Option<String>,
    value: Option<i32>,
}

#[derive(Parser)]
struct Opts {
    path: PathBuf,
}

const MAX_LENGTH: usize = 64;

fn main() -> Result<()> {
    env_logger::init();
    let opts = Opts::parse();

    if opts.path.exists() {
        std::fs::remove_file(&opts.path)?;
    }
    let socket = UnixDatagram::bind(opts.path)?;

    let mut buf = [0u8; MAX_LENGTH];
    let mut tags: HashMap<String, u32> = HashMap::new();

    loop {
        let count = socket.recv(&mut buf)?;

        log::debug!("Received message: {:?}", std::str::from_utf8(&buf[..count]));

        match serde_json::from_slice(&buf[..count]) {
            Ok(NotificationRequest { tag, body, value }) => {
                let mut notification = Notification::new();
                notification.summary(&tag);
                notification.timeout(3000);
                if let Some(body) = body {
                    notification.body(&body);
                }
                if let Some(value) = value {
                    notification.hint(Hint::CustomInt("value".into(), value));
                }

                let entry = tags.entry(tag);

                match entry {
                    Entry::Occupied(mut entry) => {
                        notification.id(*entry.get());
                        let handle = notification.show()?;
                        entry.insert(handle.id());
                    }
                    Entry::Vacant(entry) => {
                        let handle = notification.show()?;
                        entry.insert(handle.id());
                    }
                }
            }
            Err(err) => {
                log::error!("Error: {err:?}");
            }
        }
    }
}
