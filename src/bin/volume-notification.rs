use std::{collections::HashMap, os::unix::net::UnixDatagram, path::PathBuf};

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
    #[clap(long)]
    socket: PathBuf,

    #[clap(short = 't', long, default_value_t = 2000)]
    duration: i32,
}

const MAX_LENGTH: usize = 1024;

fn main() -> Result<()> {
    env_logger::init();
    let opts = Opts::parse();

    if opts.socket.exists() {
        std::fs::remove_file(&opts.socket)?;
    }
    let socket = UnixDatagram::bind(opts.socket)?;

    let mut buf = [0u8; MAX_LENGTH];
    let mut tags: HashMap<String, u32> = HashMap::new();

    loop {
        let count = socket.recv(&mut buf)?;

        log::debug!("Received message: {:?}", std::str::from_utf8(&buf[..count]));

        match serde_json::from_slice(&buf[..count]) {
            Ok(NotificationRequest { tag, body, value }) => {
                let mut notification = Notification::new();
                notification.summary(&tag);
                notification.timeout(opts.duration);
                if let Some(body) = body {
                    notification.body(&body);
                }
                if let Some(value) = value {
                    notification.hint(Hint::CustomInt("value".into(), value));
                }

                // id == 0 means that this notification should not override a previous one
                let entry = tags.entry(tag).or_insert(0);
                *entry = notification.id(*entry).show()?.id();
            }
            Err(err) => log::error!("Error: {err:?}"),
        }
    }
}
