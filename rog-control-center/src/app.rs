use std::{
    f64::consts::PI,
    io::Write,
    sync::{
        atomic::{AtomicBool, AtomicU8, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use egui::{Button, RichText};
use rog_supported::SupportedFunctions;

use crate::{
    config::Config, error::Result, get_ipc_file, page_states::PageDataStates, Page,
    RogDbusClientBlocking, SHOWING_GUI,
};

pub struct RogApp<'a> {
    pub page: Page,
    pub states: PageDataStates,
    pub supported: SupportedFunctions,
    /// Should the app begin showing the GUI
    pub begin_show_gui: Arc<AtomicBool>,
    /// Is the app GUI closed (and running in bg)
    pub running_in_bg: bool,
    // TODO: can probably just open and read whenever
    pub config: Config,
    pub asus_dbus: RogDbusClientBlocking<'a>,
    /// Oscillator in percentage
    pub oscillator: Arc<AtomicU8>,
    /// Frequency of oscillation
    pub oscillator_freq: Arc<AtomicU8>,
    /// A toggle that toggles true/false when the oscillator reaches 0
    pub oscillator_toggle: Arc<AtomicBool>,
}

impl<'a> RogApp<'a> {
    /// Called once before the first frame.
    pub fn new(
        start_closed: bool,
        config: Config,
        show_gui: Arc<AtomicBool>,
        states: PageDataStates,
        _cc: &eframe::CreationContext<'_>,
    ) -> Result<Self> {
        let (dbus, _) = RogDbusClientBlocking::new()?;
        let supported = dbus.proxies().supported().supported_functions()?;

        // Set up an oscillator to run on a thread.
        // Helpful for visual effects like colour pulse.
        let oscillator = Arc::new(AtomicU8::new(0));
        let oscillator1 = oscillator.clone();
        let oscillator_freq = Arc::new(AtomicU8::new(5));
        let oscillator_freq1 = oscillator_freq.clone();
        let oscillator_toggle = Arc::new(AtomicBool::new(false));
        let oscillator_toggle1 = oscillator_toggle.clone();
        std::thread::spawn(move || {
            let started = Instant::now();
            let mut toggled = false;
            loop {
                let time = started.elapsed();
                // 32 = slow, 16 = med, 8 = fast
                let scale = oscillator_freq1.load(Ordering::SeqCst) as f64;
                let elapsed = time.as_millis() as f64 / 10000.0;
                let tmp = ((scale * elapsed * PI).cos()).abs();
                if tmp <= 0.1 && !toggled {
                    let s = oscillator_toggle1.load(Ordering::SeqCst);
                    oscillator_toggle1.store(!s, Ordering::SeqCst);
                    toggled = true;
                } else if tmp > 0.9 {
                    toggled = false;
                }

                let tmp = (255.0 * tmp * 100.0 / 255.0) as u8;

                oscillator1.store(tmp, Ordering::SeqCst);
                std::thread::sleep(Duration::from_millis(33));
            }
        });

        Ok(Self {
            supported,
            states,
            page: Page::System,
            begin_show_gui: show_gui,
            running_in_bg: start_closed,
            config,
            asus_dbus: dbus,
            oscillator,
            oscillator_toggle,
            oscillator_freq,
        })
    }
}

impl<'a> eframe::App for RogApp<'a> {
    fn on_exit_event(&mut self) -> bool {
        if self.config.run_in_background {
            self.running_in_bg = true;
            get_ipc_file().unwrap().write_all(&[0]).unwrap();
            return false;
        }
        true
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let Self {
            begin_show_gui: should_show_gui,
            supported,
            asus_dbus: dbus,
            states,
            ..
        } = self;
        states
            .refresh_if_notfied(supported, dbus)
            .map(|repaint| {
                if repaint {
                    ctx.request_repaint();
                }
            })
            .map_err(|e| self.states.error = Some(e.to_string()))
            .ok();

        let page = self.page;

        if should_show_gui.load(Ordering::SeqCst) {
            let mut ipc_file = get_ipc_file().unwrap();
            ipc_file.write_all(&[SHOWING_GUI]).unwrap();
            should_show_gui.store(false, Ordering::SeqCst);
            frame.set_visible(true);
            self.running_in_bg = false;
        }
        if self.running_in_bg {
            // Request to draw nothing at all
            ctx.request_repaint_after(Duration::from_millis(500));
            frame.set_visible(false);
            return;
        }
        // Do all GUI display after this point

        self.top_bar(ctx, frame);
        self.side_panel(ctx);

        if let Some(err) = self.states.error.clone() {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading(RichText::new("Error!").size(28.0));

                ui.centered_and_justified(|ui| {
                    ui.label(RichText::new(format!("The error was: {:?}", err)).size(22.0));
                });
            });
            egui::TopBottomPanel::bottom("error_bar")
                .default_height(26.0)
                .show(ctx, |ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        if ui
                            .add(Button::new(RichText::new("Okay").size(20.0)))
                            .clicked()
                        {
                            self.states.error = None;
                        }
                    });
                });
        } else if page == Page::System {
            self.system_page(ctx);
        } else if page == Page::AuraEffects {
            self.aura_page(ctx);
        } else if page == Page::AnimeMatrix {
            self.anime_page(ctx);
        } else if page == Page::FanCurves {
            self.fan_curve_page(ctx);
        }
    }
}
