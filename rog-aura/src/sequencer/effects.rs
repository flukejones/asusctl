use crate::{layouts::KeyLayout, p_random, Colour, EffectState, LedType, Speed};
use serde_derive::{Deserialize, Serialize};

macro_rules! effect_state_impl {
    () => {
        fn get_colour(&self) -> Colour {
            self.colour
        }

        fn get_led_type(&self) -> LedType {
            self.led_type.clone()
        }

        /// Change the led type
        fn set_led_type(&mut self, led_type: LedType) {
            self.led_type = led_type;
        }
    };
}

macro_rules! effect_impl {
    ($($effect:ident),*) => {
        impl Effect {
            /// Get the type of LED set
            pub fn get_led_type(&self) -> LedType {
                match self {
                    $(Effect::$effect(c) => c.get_led_type(),)*
                }
            }

            /// Change the led type
            pub fn set_led_type(&mut self, led_type: LedType) {
                match self {
                    $(Effect::$effect(c) => c.set_led_type(led_type),)*
                }
            }

            /// Calculate the next state of the effect
            pub fn next_state(&mut self, layout: &KeyLayout) {
                match self {
                    $(Effect::$effect(c) => c.next_colour_state(layout),)*
                }
            }

            /// Get the calculated colour
            pub fn get_colour(&self) -> Colour {
                match self {
                    $(Effect::$effect(c) => c.get_colour(),)*
                }
            }
        }
    };
}

/**************************************************************************************************/

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Effect {
    Static(Static),
    Breathe(Breathe),
    Flicker(Flicker),
}

impl Default for Effect {
    fn default() -> Self {
        Self::Static(Static::new(LedType::default(), Colour::default()))
    }
}

effect_impl!(Static, Breathe, Flicker);

/**************************************************************************************************/

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Static {
    led_type: LedType,
    /// The starting colour
    colour: Colour,
}

impl Static {
    pub fn new(led_type: LedType, colour: Colour) -> Self {
        Self { led_type, colour }
    }
}

impl EffectState for Static {
    fn next_colour_state(&mut self, _layout: &KeyLayout) {}

    effect_state_impl!();
}

/**************************************************************************************************/

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Breathe {
    led_type: LedType,
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
    pub fn new(led_type: LedType, colour1: Colour, colour2: Colour, speed: Speed) -> Self {
        Self {
            led_type,
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

    effect_state_impl!();
}

/**************************************************************************************************/

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Flicker {
    led_type: LedType,
    start_colour: Colour,
    max_percentage: u8,
    min_percentage: u8,
    #[serde(skip)]
    count: u8,
    #[serde(skip)]
    colour: Colour,
}

impl Flicker {
    pub fn new(led_type: LedType, colour: Colour, max_percentage: u8, min_percentage: u8) -> Self {
        Self {
            led_type,
            colour,
            count: 4,
            max_percentage,
            min_percentage,
            start_colour: colour,
        }
    }
}

impl EffectState for Flicker {
    fn next_colour_state(&mut self, _layout: &KeyLayout) {
        let Self {
            max_percentage,
            min_percentage,
            colour,
            start_colour,
            ..
        } = self;

        if self.count == 0 {
            self.count = 4;
        }
        self.count -= 1;
        if self.count != 0 {
            return;
        }

        // TODO: make a "percentage" method on Colour.
        let max_light = Colour(
            (start_colour.0 as f32 / 100.0 * *max_percentage as f32) as u8,
            (start_colour.1 as f32 / 100.0 * *max_percentage as f32) as u8,
            (start_colour.2 as f32 / 100.0 * *max_percentage as f32) as u8,
        );
        // min light is a percentage of the set colour
        let min_light = Colour(
            (start_colour.0 as f32 / 100.0 * *min_percentage as f32) as u8,
            (start_colour.1 as f32 / 100.0 * *min_percentage as f32) as u8,
            (start_colour.2 as f32 / 100.0 * *min_percentage as f32) as u8,
        );

        // Convert the 255 to percentage
        let amount = (p_random() & 7) as f32 * 8.0;

        let set_colour = |colour: &mut u8, max: f32, min: f32| {
            let pc = amount / max * 100.0;
            let min_amount = pc * min / 100.0; // percentage of min colour
            let max_amount = pc * max / 100.0; // percentage of max colour
            if *colour as f32 - min_amount < min {
                *colour = min as u8;
            } else {
                *colour = (max - max_amount) as u8;
            }
        };
        set_colour(&mut colour.0, max_light.0 as f32, min_light.0 as f32);
        set_colour(&mut colour.1, max_light.1 as f32, min_light.1 as f32);
        set_colour(&mut colour.2, max_light.2 as f32, min_light.2 as f32);

        self.count = 4;
    }

    effect_state_impl!();
}
