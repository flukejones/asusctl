#![deny(unused_must_use)]
/// Configuration loading, saving
pub mod config;
/// Control of AniMe matrix display
pub mod ctrl_anime;
/// Keyboard LED brightness control, RGB, and LED display modes
pub mod ctrl_aura;
/// Control of battery charge level
pub mod ctrl_charge;
/// Control CPU min/max freq and turbo, fan mode, fan curves
///
/// Intel machines can control:
/// - CPU min/max frequency
/// - CPU turbo enable/disable
/// - Fan mode (normal, boost, silent)
///
/// AMD machines can control:
/// - CPU turbo enable/disable
/// - Fan mode (normal, boost, silent)
/// - Fan min/max RPM curve
pub mod ctrl_profiles;
/// Control ASUS bios function such as boot sound, Optimus/Dedicated gfx mode
pub mod ctrl_rog_bios;
/// Laptop matching to determine capabilities
pub mod laptops;

/// Fetch all supported functions for the laptop
pub mod ctrl_supported;

pub mod error;

use std::time::Duration;

use crate::error::RogError;
use async_trait::async_trait;
use config::Config;
use log::warn;
use smol::{stream::StreamExt, Executor, Timer};
use zbus::Connection;
use zvariant::ObjectPath;

pub static VERSION: &str = env!("CARGO_PKG_VERSION");

pub trait Reloadable {
    fn reload(&mut self) -> Result<(), RogError>;
}

#[async_trait]
pub trait ZbusAdd {
    async fn add_to_server(self, server: &mut Connection);

    async fn add_to_server_helper(
        iface: impl zbus::Interface,
        path: &str,
        server: &mut Connection,
    ) {
        server
            .object_server()
            .at(&ObjectPath::from_str_unchecked(path), iface)
            .await
            .map_err(|err| {
                warn!("{}: add_to_server {}", path, err);
                err
            })
            .ok();
    }
}

/// Set up a task to run on the async executor
#[async_trait]
pub trait CtrlTask {
    /// Implement to set up various tasks that may be required, using the `Executor`.
    /// No blocking loops are allowed, or they must be run on a separate thread.
    async fn create_tasks(&self, executor: &mut Executor) -> Result<(), RogError>;

    /// Create a timed repeating task
    async fn repeating_task(
        &self,
        millis: u64,
        executor: &mut Executor,
        mut task: impl FnMut() + Send + 'static,
    ) {
        let timer = Timer::interval(Duration::from_millis(millis));
        executor
            .spawn(async move {
                timer.for_each(|_| task()).await;
            })
            .detach();
    }
}

pub trait CtrlTaskComplex {
    type A;

    fn do_task(&mut self, config: &mut Config, event: Self::A);
}

pub trait GetSupported {
    type A;

    fn get_supported() -> Self::A;
}
