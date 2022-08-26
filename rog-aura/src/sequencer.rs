use serde_derive::{Deserialize, Serialize};

use crate::{
    keys::Key, layouts::KeyLayout, Colour, KeyColourArray, PerKeyRaw, PerZone, Speed,
    ZonedColourArray,
};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum LedType {
    Key(Key),
    Zone(PerZone),
}

impl Default for LedType {
    fn default() -> Self {
        Self::Zone(PerZone::None)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(super) enum Action {
    Static(Colour),
    Breathe {
        /// The starting colour
        colour1: Colour,
        /// The secondary starting colour
        colour2: Colour,
        /// The speed at which to cycle between the colours
        speed: Speed,
        /// Temporary data to help keep state
        #[serde(skip)]
        colour_actual: Colour,
        #[serde(skip)]
        count_flipped: bool,
        #[serde(skip)]
        use_colour1: bool,
    },
}

impl Default for Action {
    fn default() -> Self {
        Self::Static(Colour::default())
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct ActionData {
    led_type: LedType,
    action: Action,
    // TODO: time
    /// The end resulting colour after stepping through effect
    #[serde(skip)]
    colour: Colour,
}

impl ActionData {
    pub fn set_led_type(&mut self, led_type: LedType) {
        self.led_type = led_type
    }

    pub fn new_static(led_type: LedType, colour: Colour) -> Self {
        Self {
            led_type,
            action: Action::Static(colour),
            colour: Default::default(),
        }
    }

    pub fn new_breathe(led_type: LedType, colour1: Colour, colour2: Colour, speed: Speed) -> Self {
        Self {
            led_type,
            action: Action::Breathe {
                colour1,
                colour2,
                speed,
                colour_actual: colour1,
                count_flipped: false,
                use_colour1: true,
            },
            colour: Default::default(),
        }
    }

    pub fn next_state(&mut self, _layout: &KeyLayout) {
        match &mut self.action {
            Action::Static(c) => self.colour = *c,
            Action::Breathe {
                colour1,
                colour2,
                speed,
                colour_actual,
                count_flipped: flipped,
                use_colour1,
            } => {
                let speed = 4 - <u8>::from(*speed);

                let colour: &mut Colour;
                if *colour_actual == Colour(0, 0, 0) {
                    *use_colour1 = !*use_colour1;
                }

                if !*use_colour1 {
                    colour = colour2;
                } else {
                    colour = colour1;
                }

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
                self.colour = *colour_actual;
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Sequences(Vec<ActionData>);

impl Sequences {
    #[inline]
    pub fn new() -> Self {
        Self(Vec::new())
    }

    #[inline]
    pub fn push(&mut self, action: ActionData) {
        self.0.push(action);
    }

    #[inline]
    pub fn insert(&mut self, index: usize, action: ActionData) {
        self.0.insert(index, action);
    }

    /// Remove an item at this position from the run buffer. If the `index` supplied
    /// is not in range then `None` is returned, otherwise the `ActionData` at that location
    /// is yeeted and returned.
    #[inline]
    pub fn remove_item(&mut self, index: usize) -> Option<ActionData> {
        if index < self.0.len() {
            return Some(self.0.remove(index));
        }
        None
    }

    pub fn next_state(&mut self, layout: &KeyLayout) {
        for effect in self.0.iter_mut() {
            effect.next_state(layout);
        }
    }

    pub fn create_packets(&self) -> PerKeyRaw {
        let mut keys = KeyColourArray::new();
        let mut zones = ZonedColourArray::new();
        let mut is_per_key = false;
        for effect in self.0.iter() {
            match effect.led_type {
                LedType::Key(key) => {
                    is_per_key = true;
                    if let Some(rgb) = keys.rgb_for_key(key) {
                        rgb[0] = effect.colour.0;
                        rgb[1] = effect.colour.1;
                        rgb[2] = effect.colour.2;
                    }
                }
                LedType::Zone(z) => {
                    let rgb = zones.rgb_for_zone(z);
                    rgb[0] = effect.colour.0;
                    rgb[1] = effect.colour.1;
                    rgb[2] = effect.colour.2;
                }
            }
        }
        if is_per_key {
            keys.into()
        } else {
            vec![zones.into()]
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        keys::Key, layouts::KeyLayout, Action, ActionData, Colour, LedType, Sequences, Speed,
    };

    #[test]
    fn single_key_next_state_then_create() {
        let layout = KeyLayout::gx502_layout();
        let mut seq = Sequences::new();
        seq.0.push(ActionData {
            led_type: LedType::Key(Key::F),
            action: Action::Static(Colour(255, 127, 0)),
            colour: Default::default(),
        });

        seq.next_state(&layout);
        let packets = seq.create_packets();

        assert_eq!(packets[0][0], 0x5d);
        assert_eq!(packets[5][33], 255);
        assert_eq!(packets[5][34], 127);
        assert_eq!(packets[5][35], 0);
    }

    #[test]
    fn cycle_breathe() {
        let layout = KeyLayout::gx502_layout();
        let mut seq = Sequences::new();
        seq.0.push(ActionData {
            led_type: LedType::Key(Key::F),
            action: Action::Breathe {
                colour1: Colour(255, 127, 0),
                colour2: Colour(127, 0, 255),
                speed: Speed::Med,
                colour_actual: Colour(255, 127, 0),
                count_flipped: false,
                use_colour1: true,
            },
            colour: Default::default(),
        });

        let s = serde_json::to_string_pretty(&seq).unwrap();
        println!("{s}");

        seq.next_state(&layout);
        let packets = seq.create_packets();

        assert_eq!(packets[0][0], 0x5d);
        assert_eq!(packets[5][33], 213);
        assert_eq!(packets[5][34], 106);
        assert_eq!(packets[5][35], 0);

        // dbg!(&packets[5][33..=35]);

        seq.next_state(&layout);
        let packets = seq.create_packets();

        assert_eq!(packets[0][0], 0x5d);
        assert_eq!(packets[5][33], 171);
        assert_eq!(packets[5][34], 85);
        assert_eq!(packets[5][35], 0);
    }
}
