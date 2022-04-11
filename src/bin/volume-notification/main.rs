mod notify;

use std::{collections::HashMap, future::pending, sync::Arc};

use anyhow::Result;
use futures_util::{lock::Mutex, StreamExt};
use zbus::{dbus_interface, zvariant::Value, Connection};

use notify::Notification;

struct VolumeNotification {
    ids: Arc<Mutex<HashMap<String, u32>>>,
    connection: Arc<Connection>,
}

#[dbus_interface(interface = "xyz.domain_name.VolumeNotification")]
impl VolumeNotification {
    async fn notify(&mut self, tag: &str, body: &str, value: i32) -> zbus::fdo::Result<()> {
        let mut notification = Notification::new();
        notification.summary(&tag);
        notification.timeout(2000);
        if body != "" {
            notification.body(body);
        }
        if value != -1 {
            notification.add_hint("value", Value::I32(value));
        }

        let mut ids = self.ids.lock().await;
        let id = ids.entry(tag.to_owned()).or_insert(0);
        *id = notification.replaces_id(*id).show(&self.connection).await?;

        Ok(())
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    env_logger::init();

    let ids = Arc::new(Mutex::new(HashMap::new()));
    let connection = Arc::new(Connection::session().await?);

    let volume_notification_interface = VolumeNotification {
        ids: ids.clone(),
        connection: connection.clone(),
    };

    connection
        .object_server()
        .at(
            "/xyz/domain_name/VolumeNotification",
            volume_notification_interface,
        )
        .await?;

    connection
        .request_name("xyz.domain_name.VolumeNotification")
        .await?;

    tokio::task::spawn({
        let connection = connection.clone();
        let ids = ids.clone();

        async move {
            let mut stream = notify::closed(&connection).await.unwrap();

            while let Some(msg) = stream.next().await {
                let args = msg.args().unwrap();
                let id = args.id();

                ids.lock().await.retain(|_k, v| v != id)
            }
        }
    });

    pending().await
}
