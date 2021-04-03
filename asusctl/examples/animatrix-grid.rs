use rog_anime::{AniMeDataBuffer, AniMeGrid};
use rog_dbus::AuraDbusClient;

// In usable data:
// Top row start at 1, ends at 32

fn main() {
    let (client, _) = AuraDbusClient::new().unwrap();
    let mut matrix = AniMeGrid::new();
    {
        let tmp = matrix.get_mut();
        for row in tmp.iter_mut() {
            row[row.len() - 33] = 0xff;

            row[row.len() - 22] = 0xff;

            row[row.len() - 11] = 0xff;

            row[row.len() - 1] = 0xff;
        }
    }

    let matrix = <AniMeDataBuffer>::from(matrix);

    client.proxies().anime().write(matrix).unwrap();
}
