declare const global: any, imports: any;
//@ts-ignore
const ThisModule = imports.misc.extensionUtils.getCurrentExtension();

const GLib = imports.gi.GLib;

export class File {
    public static DBus(name: string) {
        let file = `${ThisModule.path}/resources/dbus/${name}.xml`;
        try {
            let [_ok, bytes] = GLib.file_get_contents(file)
            if (!_ok)
            //@ts-ignore
                log(`Couldn't read contents of "${file}"`);
            return _ok ? imports.byteArray.toString(bytes) : null;
        } catch (e) {
            //@ts-ignore
            log(`Failed to load "${file}"`, e);
        }
    }
}
