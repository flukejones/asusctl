use std::{thread::sleep, time::Duration};

use rog_anime::{AniMeDataBuffer, AniMeDiagonal};
use rog_dbus::AuraDbusClient;

// In usable data:
// Top row start at 1, ends at 32

// 74w x 36h diagonal used by the windows app

fn main() {
    let (client, _) = AuraDbusClient::new().unwrap();

    for step in (2..50).rev() {
        let mut matrix = AniMeDiagonal::new();
        for c in (0..60).into_iter().step_by(step) {
            for i in matrix.get_mut().iter_mut() {
                i[c] = 50;
            }
        }

        for c in (0..35).into_iter().step_by(step) {
            for i in matrix.get_mut()[c].iter_mut() {
                *i = 50;
            }
        }

        let m = <AniMeDataBuffer>::from(&matrix);
        client.proxies().anime().write(m).unwrap();
        sleep(Duration::from_millis(300));
    }
}
