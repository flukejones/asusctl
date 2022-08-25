use serde_derive::{Deserialize, Serialize};

use crate::{keys::Key, Colour, KeyColourArray, PerKeyRaw, Speed};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct PerKey {
    pub key: Key,
    action: ActionData,
    /// The end resulting colour after stepping through effect
    #[serde(skip)]
    colour: Colour,
}

impl PerKey {
    pub fn new_breathe(key: Key, colour1: Colour, colour2: Colour, speed: Speed) -> Self {
        Self {
            key,
            action: ActionData::new_breathe(colour1, colour2, speed),
            colour: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(super) enum ActionData {
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

impl ActionData {
    fn new_breathe(colour1: Colour, colour2: Colour, speed: Speed) -> Self {
        Self::Breathe {
            colour1,
            colour2,
            speed,
            colour_actual: colour1,
            count_flipped: false,
            use_colour1: true,
        }
    }
}

impl Default for ActionData {
    fn default() -> Self {
        Self::Static(Colour::default())
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Sequences(Vec<PerKey>);

impl Sequences {
    #[inline]
    pub fn new() -> Self {
        Self(Vec::new())
    }

    #[inline]
    pub fn push(&mut self, action: PerKey) {
        self.0.push(action);
    }

    #[inline]
    pub fn insert(&mut self, index: usize, action: PerKey) {
        self.0.insert(index, action);
    }

    /// Remove an item at this position from the run buffer. If the `index` supplied
    /// is not in range then `None` is returned, otherwise the `ActionData` at that location
    /// is yeeted and returned.
    #[inline]
    pub fn remove_item(&mut self, index: usize) -> Option<PerKey> {
        if index < self.0.len() {
            return Some(self.0.remove(index));
        }
        None
    }

    pub fn next_state(&mut self) {
        for effect in self.0.iter_mut() {
            match &mut effect.action {
                ActionData::Static(c) => effect.colour = *c,
                ActionData::Breathe {
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
                    effect.colour = *colour_actual;
                }
            }
        }
    }

    pub fn create_packets(&self) -> PerKeyRaw {
        let mut keys = KeyColourArray::new();
        for effect in self.0.iter() {
            if let Some(rgb) = keys.rgb_for_key(effect.key) {
                rgb[0] = effect.colour.0;
                rgb[1] = effect.colour.1;
                rgb[2] = effect.colour.2;
            }
        }
        keys.into()
    }
}

#[cfg(test)]
mod tests {
    use crate::{keys::Key, ActionData, Colour, PerKey, Sequences, Speed};

    #[test]
    fn single_key_next_state_then_create() {
        let mut seq = Sequences::new();
        seq.0.push(PerKey {
            key: Key::F,
            action: ActionData::Static(Colour(255, 127, 0)),
            colour: Default::default(),
        });

        seq.next_state();
        let packets = seq.create_packets();

        assert_eq!(packets[0][0], 0x5d);
        assert_eq!(packets[5][33], 255);
        assert_eq!(packets[5][34], 127);
        assert_eq!(packets[5][35], 0);
    }

    #[test]
    fn cycle_breathe() {
        let mut seq = Sequences::new();
        seq.0.push(PerKey {
            key: Key::F,
            action: ActionData::new_breathe(Colour(255, 127, 0), Colour(127, 0, 255), Speed::Med),
            colour: Default::default(),
        });

        seq.next_state();
        let packets = seq.create_packets();

        assert_eq!(packets[0][0], 0x5d);
        assert_eq!(packets[5][33], 213);
        assert_eq!(packets[5][34], 106);
        assert_eq!(packets[5][35], 0);

        // dbg!(&packets[5][33..=35]);

        seq.next_state();
        let packets = seq.create_packets();

        assert_eq!(packets[0][0], 0x5d);
        assert_eq!(packets[5][33], 171);
        assert_eq!(packets[5][34], 85);
        assert_eq!(packets[5][35], 0);
    }
}
