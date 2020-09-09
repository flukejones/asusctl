const INIT_STR: &str = "ASUS Tech.Inc.";
const PACKET_SIZE: usize = 640;

// Only these two packets must be 17 bytes
const DEV_PAGE: u8 = 0x5e;
// These bytes are in [1] position of the array
const WRITE: u8 = 0xc0;
const INIT: u8 = 0xc2;
const APPLY: u8 = 0xc3;
const SET: u8 = 0xc4;

use crate::config::Config;
use asus_nb::error::AuraError;
use log::{error, info, warn};
use rusb::{Device, DeviceHandle};
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::Receiver;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

#[allow(dead_code)]
#[derive(Debug)]
pub enum AnimatrixCommand {
    Apply,
    Set,
    WriteImage(Vec<Vec<u8>>),
    //ReloadLast,
}

pub struct CtrlAnimeDisplay {
    handle: DeviceHandle<rusb::GlobalContext>,
    initialised: bool,
}

use ::dbus::{nonblock::SyncConnection, tree::Signal};
use async_trait::async_trait;

#[async_trait]
impl crate::Controller for CtrlAnimeDisplay {
    type A = Vec<Vec<u8>>;

    /// Spawns two tasks which continuously check for changes
    fn spawn_task_loop(
        mut self,
        _: Arc<Mutex<Config>>,
        mut recv: Receiver<Self::A>,
        _: Option<Arc<SyncConnection>>,
        _: Option<Arc<Signal<()>>>,
    ) -> Vec<JoinHandle<()>> {
        vec![tokio::spawn(async move {
            while let Some(image) = recv.recv().await {
                self.do_command(AnimatrixCommand::WriteImage(image))
                    .await
                    .unwrap_or_else(|err| warn!("{}", err));
            }
        })]
    }

    async fn reload_from_config(&mut self, _: &mut Config) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

impl CtrlAnimeDisplay {
    #[inline]
    pub fn new() -> Result<CtrlAnimeDisplay, Box<dyn Error>> {
        // We don't expect this ID to ever change
        let device = CtrlAnimeDisplay::get_device(0x0b05, 0x193b)?;

        let mut device = device.open()?;
        device.reset()?;

        device.set_auto_detach_kernel_driver(true).map_err(|err| {
            error!("Auto-detach kernel driver failed: {}", err);
            err
        })?;

        device.claim_interface(0).map_err(|err| {
            error!("Could not claim device interface: {}", err);
            err
        })?;

        info!("Device has an AniMe Matrix display");
        Ok(CtrlAnimeDisplay {
            handle: device,
            initialised: false,
        })
    }

    #[inline]
    fn get_device(vendor: u16, product: u16) -> Result<Device<rusb::GlobalContext>, rusb::Error> {
        for device in rusb::devices()?.iter() {
            let device_desc = device.device_descriptor()?;
            if device_desc.vendor_id() == vendor && device_desc.product_id() == product {
                return Ok(device);
            }
        }
        Err(rusb::Error::NoDevice)
    }

    pub async fn do_command(&mut self, command: AnimatrixCommand) -> Result<(), AuraError> {
        if !self.initialised {
            self.do_initialization().await?
        }

        match command {
            AnimatrixCommand::WriteImage(effect) => self.write_image(effect).await?,
            AnimatrixCommand::Set => self.do_set().await?,
            AnimatrixCommand::Apply => self.do_apply().await?,
            //AnimatrixCommand::ReloadLast => self.reload_last_builtin(&config).await?,
        }
        Ok(())
    }

    /// Should only be used if the bytes you are writing are verified correct
    #[inline]
    async fn write_bytes(&self, message: &[u8]) -> Result<(), AuraError> {
        match self.handle.write_control(
            0x21,  // request_type
            0x09,  // request
            0x35e, // value
            0x00,  // index
            message,
            Duration::from_millis(200),
        ) {
            Ok(_) => {}
            Err(err) => match err {
                rusb::Error::Timeout => {}
                _ => error!("Failed to write to led interrupt: {}", err),
            },
        }
        Ok(())
    }

    /// Write an Animatrix image
    ///
    /// The expected input here is *two* Vectors, 640 bytes in length. The two vectors
    /// are each one half of the full image write.
    ///
    /// After each write a flush is written, it is assumed that this tells the device to
    /// go ahead and display the written bytes
    ///
    /// # Note:
    /// The vectors are expected to contain the full sequence of bytes as follows
    ///
    /// - Write pane 1: 0x5e 0xc0 0x02 0x01 0x00 0x73 0x02 .. <led brightness>
    /// - Write pane 2: 0x5e 0xc0 0x02 0x74 0x02 0x73 0x02 .. <led brightness>
    ///
    /// Where led brightness is 0..255, low to high
    #[inline]
    async fn write_image(&mut self, image: Vec<Vec<u8>>) -> Result<(), AuraError> {
        for row in image.iter() {
            self.write_bytes(row).await?;
        }
        self.do_flush().await?;
        Ok(())
    }

    #[inline]
    async fn do_initialization(&mut self) -> Result<(), AuraError> {
        let mut init = [0; PACKET_SIZE];
        init[0] = DEV_PAGE; // This is the USB page we're using throughout
        for (idx, byte) in INIT_STR.as_bytes().iter().enumerate() {
            init[idx + 1] = *byte
        }
        self.write_bytes(&init).await?;

        // clear the init array and write other init message
        for ch in init.iter_mut() {
            *ch = 0;
        }
        init[0] = DEV_PAGE; // write it to be sure?
        init[1] = INIT;

        self.write_bytes(&init).await?;
        self.initialised = true;
        Ok(())
    }

    #[inline]
    async fn do_flush(&mut self) -> Result<(), AuraError> {
        let mut flush = [0; PACKET_SIZE];
        flush[0] = DEV_PAGE;
        flush[1] = WRITE;
        flush[2] = 0x03;

        self.write_bytes(&flush).await?;
        Ok(())
    }

    #[inline]
    async fn do_set(&mut self) -> Result<(), AuraError> {
        let mut flush = [0; PACKET_SIZE];
        flush[0] = DEV_PAGE;
        flush[1] = SET;
        flush[2] = 0x01;
        flush[3] = 0x80;

        self.write_bytes(&flush).await?;
        Ok(())
    }

    #[inline]
    async fn do_apply(&mut self) -> Result<(), AuraError> {
        let mut flush = [0; PACKET_SIZE];
        flush[0] = DEV_PAGE;
        flush[1] = APPLY;
        flush[2] = 0x01;
        flush[3] = 0x80;

        self.write_bytes(&flush).await?;
        Ok(())
    }
}
