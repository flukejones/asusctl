import GLib from 'gi://GLib';
import * as AsusExtension from '../extension.js';

export class File {
    public static DBus(name: string) {
        const file = `${AsusExtension.extension.path}/resources/dbus/${name}.xml`;
        try {
            const [_ok, bytes] = GLib.file_get_contents(file);
            if (!_ok)
            //@ts-ignore
                log(`Couldn't read contents of "${file}"`);

            const decoder = new TextDecoder();
            return _ok ? decoder.decode(bytes) : null;
        } catch (e) {
            //@ts-ignore
            log(`Failed to load "${file}"`, e);
        }
    }
}
