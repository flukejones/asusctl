use std::sync::atomic::Ordering;
use std::time::Duration;

use egui::Color32;
use rog_aura::{AuraEffect, AuraModeNum};

use crate::system_state::SystemState;
use crate::widgets::{aura_modes_group, keyboard};
use crate::RogApp;

impl RogApp {
    pub fn aura_page(&mut self, states: &mut SystemState, ctx: &eframe::egui::Context) {
        let Self {
            oscillator1,
            oscillator2,
            oscillator3,
            oscillator_freq,
            ..
        } = self;

        let red = oscillator1.load(Ordering::SeqCst) as u32;
        let green = oscillator2.load(Ordering::SeqCst) as u32;
        let blue = oscillator3.load(Ordering::SeqCst) as u32;
        states.aura.nudge_wave(red as u8, green as u8, blue as u8);
        // let osc = c.r * 255 / osc;
        let c1 = states
            .aura
            .modes
            .get(&states.aura.current_mode)
            .unwrap_or(&AuraEffect::default())
            .colour1;

        let c2 = states
            .aura
            .modes
            .get(&states.aura.current_mode)
            .unwrap_or(&AuraEffect::default())
            .colour2;

        let mut colour = Color32::from_rgb(c1.r, c1.g, c1.b);
        if states.aura.current_mode == AuraModeNum::Pulse {
            colour = Color32::from_rgb(
                (red * c1.r as u32 / 100) as u8,
                (red * c1.g as u32 / 100) as u8,
                (red * c1.b as u32 / 100) as u8,
            );
        } else if states.aura.current_mode == AuraModeNum::Breathe {
            if self.oscillator_toggle.load(Ordering::SeqCst) {
                colour = Color32::from_rgb(
                    (red * c2.r as u32 / 100) as u8,
                    (red * c2.g as u32 / 100) as u8,
                    (red * c2.b as u32 / 100) as u8,
                );
            } else {
                colour = Color32::from_rgb(
                    (red * c1.r as u32 / 100) as u8,
                    (red * c1.g as u32 / 100) as u8,
                    (red * c1.b as u32 / 100) as u8,
                );
            }
        } else if states.aura.current_mode == AuraModeNum::Strobe {
            colour = Color32::from_rgb(
                (red * 255 / 100) as u8,
                (green * 255 / 100) as u8,
                (blue * 255 / 100) as u8,
            );
        }

        // TODO: animation of colour changes/periods/blending
        egui::CentralPanel::default().show(ctx, |ui| {
            aura_modes_group(states, oscillator_freq, ui);
            keyboard(
                ui,
                &states.aura_creation.keyboard_layout,
                &mut states.aura,
                colour,
            );
        });

        // Only do repaint request if on this page
        ctx.request_repaint_after(Duration::from_millis(33));
    }
}
