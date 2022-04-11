use anyhow::Result;
use clap::Parser;
use zbus::{blocking::Connection, dbus_proxy};

#[dbus_proxy(
    interface = "xyz.domain_name.VolumeNotification",
    default_service = "xyz.domain_name.VolumeNotification",
    default_path = "/xyz/domain_name/VolumeNotification"
)]
trait VolumeNotification {
    fn notify(&self, tag: &str, body: &str, value: i32) -> zbus::Result<()>;
}

#[derive(Parser)]
struct Opts {
    #[clap(long)]
    tag: String,
    #[clap(long)]
    body: Option<String>,
    #[clap(long)]
    value: Option<i32>,
}

fn main() -> Result<()> {
    let opts = Opts::parse();

    let connection = Connection::session()?;
    let volume_notification_proxy = VolumeNotificationProxyBlocking::new(&connection)?;

    volume_notification_proxy.notify(
        &opts.tag,
        opts.body.as_deref().unwrap_or(""),
        opts.value.unwrap_or(-1),
    )?;

    Ok(())
}
