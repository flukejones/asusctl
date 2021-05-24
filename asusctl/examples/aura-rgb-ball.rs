use rog_aura::{GX502Layout, Key, KeyColourArray, KeyLayout};
use rog_dbus::RogDbusClient;
use std::collections::LinkedList;

#[derive(Debug, Clone)]
struct Ball {
    position: (i32, i32),
    direction: (i32, i32),
    trail: LinkedList<(i32, i32)>,
}
impl Ball {
    fn new(x: i32, y: i32, trail_len: u32) -> Self {
        let mut trail = LinkedList::new();
        for _ in 1..=trail_len {
            trail.push_back((x, y));
        }

        Ball {
            position: (x, y),
            direction: (1, 1),
            trail,
        }
    }

    #[allow(clippy::if_same_then_else)]
    fn update(&mut self, key_map: &[[Key; 17]]) {
        let pos = self.position;
        let dir = self.direction;

        if pos.0 + dir.0 > key_map[pos.1 as usize].len() as i32 - 1 || pos.0 + dir.0 < 0 {
            self.direction.0 *= -1;
        } else if key_map[(pos.1) as usize][(pos.0 + dir.0) as usize] == Key::None {
            self.direction.0 *= -1;
        }

        if pos.1 + dir.1 > key_map.len() as i32 - 1 || pos.1 + dir.1 < 0 {
            self.direction.1 *= -1;
        } else if key_map[(pos.1 + dir.1) as usize][(pos.0) as usize] == Key::None {
            self.direction.1 *= -1;
        }

        self.trail.pop_front();
        self.trail.push_back(self.position);

        self.position.0 += self.direction.0;
        self.position.1 += self.direction.1;

        if self.position.0 > key_map[self.position.1 as usize].len() as i32 {
            self.position.0 = key_map[self.position.1 as usize].len() as i32 - 1;
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (dbus, _) = RogDbusClient::new()?;

    let mut colours = KeyColourArray::new();

    let layout = GX502Layout::default();

    let mut balls = [Ball::new(2, 1, 12), Ball::new(4, 6, 12)];

    dbus.proxies().led().init_effect()?;

    let rows = layout.get_rows();
    loop {
        for (n, ball) in balls.iter_mut().enumerate() {
            ball.update(rows);
            for (i, pos) in ball.trail.iter().enumerate() {
                if let Some(c) = colours.key(rows[pos.1 as usize][pos.0 as usize]) {
                    *c.0 = 0;
                    *c.1 = 0;
                    *c.2 = 0;
                    if n == 0 {
                        *c.0 = i as u8 * (255 / ball.trail.len() as u8);
                    } else if n == 1 {
                        *c.1 = i as u8 * (255 / ball.trail.len() as u8);
                    } else if n == 2 {
                        *c.2 = i as u8 * (255 / ball.trail.len() as u8);
                    }
                };
            }

            if let Some(c) = colours.key(rows[ball.position.1 as usize][ball.position.0 as usize]) {
                *c.0 = 255;
                *c.1 = 255;
                *c.2 = 255;
            };
        }
        dbus.proxies().led().set_per_key(&colours)?;

        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
