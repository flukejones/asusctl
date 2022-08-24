use serde_derive::{Deserialize, Serialize};

use crate::{keys::Key, Colour, KeyColourArray, PerKeyRaw, Speed};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PerKey {
    pub key: Key,
    pub action: ActionData,
    /// The end resulting colour after stepping through effect
    #[serde(skip)]
    colour: Colour,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum ActionData {
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
        colour1_actual: Colour,
        /// Temporary data to help keep state
        #[serde(skip)]
        colour2_actual: Colour,
    },
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
            match effect.action {
                ActionData::Static(c) => effect.colour = c,
                ActionData::Breathe {
                    colour1,
                    colour2,
                    speed,
                    colour1_actual,
                    colour2_actual,
                } => {
                    effect.colour = colour1;
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
    use crate::{keys::Key, Colour, PerKey, Sequences};

    #[test]
    fn single_key_next_state_then_create() {
        let mut seq = Sequences::new();
        seq.0.push(PerKey {
            key: Key::F,
            action: crate::ActionData::Static(Colour(255, 127, 0)),
            colour: Default::default(),
        });

        seq.next_state();
        let packets = seq.create_packets();

        assert_eq!(packets[0][0], 0x5d);
        assert_eq!(packets[5][33], 255);
        assert_eq!(packets[5][34], 127);
        assert_eq!(packets[5][35], 0);
    }
}
