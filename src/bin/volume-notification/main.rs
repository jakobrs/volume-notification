mod notify;

use std::{cell::RefCell, collections::HashMap, path::PathBuf, rc::Rc};

use anyhow::Result;
use async_std::{os::unix::net::UnixDatagram, prelude::StreamExt};
use clap::Parser;
use serde::Deserialize;
use zbus::{zvariant::Value, Connection};

use notify::Notification;

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

#[async_std::main]
async fn main() -> Result<()> {
    env_logger::init();
    let opts = Opts::parse();

    if opts.socket.exists() {
        std::fs::remove_file(&opts.socket)?;
    }
    let socket = UnixDatagram::bind(opts.socket).await?;

    let mut buf = [0u8; MAX_LENGTH];
    let tags: Rc<RefCell<HashMap<String, u32>>> = Rc::new(RefCell::new(HashMap::new()));

    let connection = Rc::new(Connection::session().await?);

    {
        let connection = connection.clone();
        let tags = tags.clone();
        async_std::task::spawn_local(async move {
            let mut stream = notify::closed(&connection).await.unwrap();

            while let Some(msg) = stream.next().await {
                let args = msg.args().unwrap();

                tags.borrow_mut().retain(|_k, v| v != args.id())
            }
        });
    }

    loop {
        let count = socket.recv(&mut buf).await?;

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
                    notification.add_hint("value", Value::I32(value));
                }

                // cannot use the entry API because of the await point
                let &old_id = tags.borrow().get(&tag).unwrap_or(&0);
                let id = notification.replaces_id(old_id).show(&connection).await?;
                tags.borrow_mut().insert(tag, id);
            }
            Err(err) => {
                log::error!("Error: {err:?}");
            }
        }
    }
}
