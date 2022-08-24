//! Very bad rushed example. The better way to do this would be to have
//! the balles move on their own square grid, then translate that to the
//! key layout via shape by pitch etc.
use rog_aura::{
    layouts::{KeyLayout, KeyRow},
    KeyColourArray,
};
use rog_dbus::RogDbusClientBlocking;
use std::collections::LinkedList;

#[derive(Debug, Clone)]
struct Ball {
    position: (f32, f32),
    direction: (f32, f32),
    trail: LinkedList<(f32, f32)>,
}
impl Ball {
    fn new(x: f32, y: f32, trail_len: u32) -> Self {
        let mut trail = LinkedList::new();
        for _ in 1..=trail_len {
            trail.push_back((x, y));
        }

        Ball {
            position: (x, y),
            direction: (1.0, 1.0),
            trail,
        }
    }

    #[allow(clippy::if_same_then_else)]
    fn update(&mut self, key_map: &[KeyRow]) {
        self.position.0 += self.direction.0;
        self.position.1 += self.direction.1;

        if self.position.1.abs() as usize >= key_map.len() {
            self.direction.1 *= -1.0;
            self.position.1 += self.direction.1;
            self.direction.0 *= -1.0;
            self.position.0 += self.direction.0;
        }
        if self.position.0.abs() as usize >= key_map[self.position.1.abs() as usize].row_ref().len()
        {
            self.direction.1 *= -1.0;
            self.position.1 += self.direction.1;
        }
        if self.position.0 as usize >= key_map[self.position.1.abs() as usize].row_ref().len() {
            self.direction.0 *= -1.0;
            self.position.0 += self.direction.0;
        }

        let pos = self.position;

        if pos.1 == key_map[pos.1.abs() as usize].row_ref().len() as f32 - 1.0 || pos.1 <= 0.0 {
            self.direction.0 *= -1.0;
        } else if key_map[(pos.1) as usize].row_ref()[(pos.0) as usize].is_placeholder() {
            self.direction.0 *= -1.0;
        }

        if pos.0 == key_map.len() as f32 - 1.0 || pos.0 <= 0.0 {
            self.direction.1 *= -1.0;
        } else if key_map[(pos.1) as usize].row_ref()[(pos.0) as usize].is_placeholder() {
            self.direction.1 *= -1.0;
        }

        self.trail.pop_front();
        self.trail.push_back(self.position);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (dbus, _) = RogDbusClientBlocking::new()?;

    let mut colours = KeyColourArray::new();
    let layout = KeyLayout::gx502_layout();

    let mut balls = [Ball::new(2.0, 1.0, 12), Ball::new(5.0, 2.0, 12)];
    // let mut balls = [Ball::new(2, 1, 12)];

    loop {
        for (n, ball) in balls.iter_mut().enumerate() {
            ball.update(layout.rows_ref());
            for (i, pos) in ball.trail.iter().enumerate() {
                if let Some(c) = colours
                    .rgb_for_key(layout.rows_ref()[pos.1.abs() as usize].row_ref()[pos.0 as usize])
                {
                    c[0] = 0;
                    c[1] = 0;
                    c[2] = 0;
                    if n == 0 {
                        c[0] = i as u8 * (255 / ball.trail.len() as u8);
                    } else if n == 1 {
                        c[1] = i as u8 * (255 / ball.trail.len() as u8);
                    } else if n == 2 {
                        c[2] = i as u8 * (255 / ball.trail.len() as u8);
                    }
                };
            }

            if let Some(c) = colours.rgb_for_key(
                layout.rows_ref()[ball.position.1.abs() as usize].row_ref()
                    [ball.position.0 as usize],
            ) {
                c[0] = 255;
                c[1] = 255;
                c[2] = 255;
            };
        }
        dbus.proxies().led().per_key_raw(colours.get())?;

        std::thread::sleep(std::time::Duration::from_millis(150));
    }
}
