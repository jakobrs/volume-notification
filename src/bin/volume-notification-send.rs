use std::{os::unix::net::UnixDatagram, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use serde::Serialize;

#[derive(Serialize)]
struct NotificationRequest {
    tag: String,
    body: Option<String>,
    value: Option<i32>,
}

#[derive(Parser)]
struct Opts {
    #[clap(long)]
    tag: String,
    #[clap(long)]
    body: Option<String>,
    #[clap(long)]
    value: Option<i32>,

    #[clap(long)]
    socket: PathBuf,
}

fn main() -> Result<()> {
    let opts = Opts::parse();
    let notification_request = NotificationRequest {
        tag: opts.tag,
        body: opts.body,
        value: opts.value,
    };
    let notification_request_json = serde_json::to_string(&notification_request)?;

    let socket = UnixDatagram::unbound()?;
    socket.send_to(notification_request_json.as_bytes(), &opts.socket)?;

    Ok(())
}
