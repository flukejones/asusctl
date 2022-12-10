use super::{EffectState, InputForEffect};
use crate::advanced::LedCode;
use crate::Colour;

pub struct InputBased {
    address: LedCode,
    colour: Colour,
    /// - audio
    /// - cpu freq
    /// - temperature
    /// - fan speed
    /// - time
    input: Box<dyn InputForEffect>,
}

impl EffectState for InputBased {
    fn next_colour_state(&mut self, _layout: &crate::layouts::KeyLayout) {
        self.input.next_colour_state();
        self.colour = self.input.get_colour();
    }

    fn get_colour(&self) -> Colour {
        self.colour
    }

    fn get_led(&self) -> LedCode {
        self.address
    }

    fn set_led(&mut self, address: LedCode) {
        self.address = address
    }
}
