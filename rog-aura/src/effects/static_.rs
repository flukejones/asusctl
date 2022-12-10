use serde::{Deserialize, Serialize};

use super::EffectState;
use crate::advanced::LedCode;
use crate::layouts::KeyLayout;
use crate::{effect_state_impl, Colour};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Static {
    address: LedCode,
    /// The starting colour
    colour: Colour,
}

impl Static {
    pub fn new(address: LedCode, colour: Colour) -> Self {
        Self { address, colour }
    }
}

impl EffectState for Static {
    effect_state_impl!();

    fn next_colour_state(&mut self, _layout: &KeyLayout) {}
}
