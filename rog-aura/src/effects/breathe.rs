use serde::{Deserialize, Serialize};

use super::EffectState;
use crate::advanced::LedCode;
use crate::layouts::KeyLayout;
use crate::{effect_state_impl, Colour, Speed};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Breathe {
    led: LedCode,
    /// The starting colour
    start_colour1: Colour,
    /// The secondary starting colour
    start_colour2: Colour,
    /// The speed at which to cycle between the colours
    speed: Speed,
    /// Temporary data to help keep state
    #[serde(skip)]
    colour: Colour,
    #[serde(skip)]
    count_flipped: bool,
    #[serde(skip)]
    use_colour1: bool,
}

impl Breathe {
    pub fn new(address: LedCode, colour1: Colour, colour2: Colour, speed: Speed) -> Self {
        Self {
            led: address,
            start_colour1: colour1,
            start_colour2: colour2,
            speed,
            colour: colour1,
            count_flipped: false,
            use_colour1: true,
        }
    }
}

impl EffectState for Breathe {
    effect_state_impl!();

    fn next_colour_state(&mut self, _layout: &KeyLayout) {
        let Self {
            start_colour1: colour1,
            start_colour2: colour2,
            speed,
            colour: colour_actual,
            count_flipped: flipped,
            use_colour1,
            ..
        } = self;

        let speed = 4 - <u8>::from(*speed);

        if *colour_actual == Colour(0, 0, 0) {
            *use_colour1 = !*use_colour1;
        }

        let colour = if !*use_colour1 { colour2 } else { colour1 };

        let r1_scale = colour.0 / speed / 2;
        let g1_scale = colour.1 / speed / 2;
        let b1_scale = colour.2 / speed / 2;

        if *colour_actual == Colour(0, 0, 0) {
            *flipped = true;
        } else if colour_actual >= colour {
            *flipped = false;
        }

        if !*flipped {
            colour_actual.0 = colour_actual.0.saturating_sub(r1_scale);
            colour_actual.1 = colour_actual.1.saturating_sub(g1_scale);
            colour_actual.2 = colour_actual.2.saturating_sub(b1_scale);
        } else {
            colour_actual.0 = colour_actual.0.saturating_add(r1_scale);
            colour_actual.1 = colour_actual.1.saturating_add(g1_scale);
            colour_actual.2 = colour_actual.2.saturating_add(b1_scale);
        }
    }
}
