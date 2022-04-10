use std::collections::HashMap;

use zbus::{dbus_proxy, zvariant::Value, Connection};

#[dbus_proxy(
    interface = "org.freedesktop.Notifications",
    default_service = "org.freedesktop.Notifications",
    default_path = "/org/freedesktop/Notifications"
)]
trait Notifications {
    fn notify(
        &self,
        app_name: &str,
        replaces_id: u32,
        app_icon: &str,
        summary: &str,
        body: &str,
        actions: &[&str],
        hints: &HashMap<&str, &Value<'_>>,
        expire_timeout: i32,
    ) -> zbus::Result<u32>;

    #[dbus_proxy(signal)]
    fn notification_closed(&self, id: u32, reason: u32) -> zbus::Result<()>;
}

#[derive(Default)]
pub struct Notification {
    app_name: String,
    replaces_id: u32,
    app_icon: String,
    summary: String,
    body: String,
    actions: Vec<String>,
    hints: HashMap<String, Value<'static>>,
    expire_timeout: i32,
}

impl Notification {
    pub fn new() -> Notification {
        Default::default()
    }

    pub fn summary(&mut self, summary: &str) -> &mut Self {
        self.summary = summary.to_owned();
        self
    }

    pub fn replaces_id(&mut self, id: u32) -> &mut Self {
        self.replaces_id = id;
        self
    }

    pub fn body(&mut self, body: &str) -> &mut Self {
        self.body = body.to_owned();
        self
    }

    pub fn timeout(&mut self, timeout: i32) -> &mut Self {
        self.expire_timeout = timeout;
        self
    }

    pub fn add_hint(&mut self, key: &str, value: Value<'static>) -> &mut Self {
        self.hints.insert(key.to_owned(), value);
        self
    }

    pub async fn show(&self, connection: &Connection) -> zbus::Result<u32> {
        let proxy = NotificationsProxy::new(connection).await?;

        let id = proxy
            .notify(
                &self.app_name,
                self.replaces_id,
                &self.app_icon,
                &self.summary,
                &self.body,
                &self
                    .actions
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<&str>>(),
                &self
                    .hints
                    .iter()
                    .map(|(k, v)| (k.as_str(), v))
                    .collect::<HashMap<&str, &Value<'_>>>(),
                self.expire_timeout,
            )
            .await?;

        Ok(id)
    }
}

pub async fn closed(connection: &Connection) -> zbus::Result<NotificationClosedStream<'_>> {
    let proxy = NotificationsProxy::new(connection).await?;

    let stream = proxy.receive_notification_closed().await?;

    Ok(stream)
}
