mod notify;

use std::{cell::RefCell, collections::HashMap, path::PathBuf, rc::Rc};

use anyhow::Result;
use bincode::Options;
use byte_string::ByteStr;
use clap::Parser;
use futures_util::StreamExt;
use serde::Deserialize;
use tokio::{
    net::UnixDatagram,
    task::{JoinHandle, LocalSet},
};
use zbus::{zvariant::Value, Connection};

use notify::Notification;

#[derive(Deserialize, Debug)]
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

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    env_logger::init();
    let opts = Opts::parse();

    let ids = Rc::new(RefCell::new(HashMap::new()));
    let connection = Rc::new(Connection::session().await?);

    let local_set = LocalSet::new();

    local_set.spawn_local({
        let connection = connection.clone();
        let ids = ids.clone();

        async move {
            let mut stream = notify::closed(&connection).await.unwrap();

            while let Some(msg) = stream.next().await {
                let args = msg.args().unwrap();
                let id = args.id();

                log::debug!("Notification closed");

                ids.borrow_mut().retain(|_k, v| v != id)
            }
        }
    });

    let main_loop: JoinHandle<Result<()>> = local_set.spawn_local(async move {
        if opts.socket.exists() {
            std::fs::remove_file(&opts.socket)?;
        }
        let socket = UnixDatagram::bind(opts.socket)?;

        let bincode_options = bincode::DefaultOptions::new();

        let mut buf = [0u8; MAX_LENGTH];
        loop {
            let count = socket.recv(&mut buf).await?;

            log::debug!("Received message: {:?}", ByteStr::new(&buf[..count]));

            match bincode_options.deserialize(&buf[..count]) {
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
                    let &old_id = ids.borrow().get(&tag).unwrap_or(&0);
                    let id = notification.replaces_id(old_id).show(&connection).await?;
                    ids.borrow_mut().insert(tag, id);
                }
                Err(err) => {
                    log::error!("Error: {err:?}");
                }
            }
        }
    });

    local_set.run_until(main_loop).await??;

    Ok(())
}
