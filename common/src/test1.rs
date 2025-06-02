use zbus::object_server::SignalEmitter;
use zbus::{Result, interface, proxy};

#[proxy]
pub trait Test1 {
    fn activate(&self) -> Result<()>;
    fn deactivate(&self) -> Result<()>;

    #[zbus(property)]
    fn state(&self) -> Result<String>;

    #[zbus(signal)]
    fn heartbeat(&self, timestamp: u64) -> Result<()>;
}

pub struct Test1 {
    pub state: String,
}

#[interface(name = "org.zbus.Test1")]
impl Test1 {
    async fn activate(
        &mut self,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
    ) -> zbus::fdo::Result<()> {
        self.state = "active".into();
        self.state_changed(&emitter).await?;
        Ok(())
    }

    async fn deactivate(
        &mut self,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
    ) -> zbus::fdo::Result<()> {
        self.state = "idle".into();
        self.state_changed(&emitter).await?;
        Ok(())
    }

    #[zbus(property)]
    async fn state(&self) -> &str {
        &self.state
    }

    #[zbus(signal)]
    async fn heartbeat(emitter: SignalEmitter<'_>, timestamp: u64) -> Result<()>;
}
