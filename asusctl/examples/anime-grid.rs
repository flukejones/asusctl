use rog_anime::{AnimeDataBuffer, AnimeGrid};
use rog_dbus::AuraDbusClient;

// In usable data:
// Top row start at 1, ends at 32

// 74w x 36h diagonal used by the windows app

fn main() {
    let (client, _) = AuraDbusClient::new().unwrap();
    let mut matrix = AnimeGrid::new(None);
    let tmp = matrix.get_mut();

    let mut i = 0;
    for (y, row) in tmp.iter_mut().enumerate() {
        if y % 2 == 0 && i + 1 != row.len() - 1 {
            i += 1;
            dbg!(i);
        }
        row[row.len() - i] = 0x22;
        if i > 5 {
            row[row.len() - i + 5] = 0x22;
        }
        if i > 10 {
            row[row.len() - i + 10] = 0x22;
        }

        if i > 15 {
            row[row.len() - i + 15] = 0x22;
        }

        if i > 20 {
            row[row.len() - i + 20] = 0x22;
        }

        if i > 25 {
            row[row.len() - i + 25] = 0x22;
        }
    }

    let matrix = <AnimeDataBuffer>::from(matrix);

    client.proxies().anime().write(matrix).unwrap();
}
