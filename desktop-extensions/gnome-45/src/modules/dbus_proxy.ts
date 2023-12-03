import { Extension, gettext as _ } from "@girs/gnome-shell/extensions/extension";
import { Gio } from "@girs/gio-2.0";
import { GLib } from "@girs/glib-2.0";
import { imports } from "@girs/gjs";

// Reads the contents of a file contained in the global resources archive. The data
// is returned as a string.
export function getStringResource(path: string | null) {
  const data = Gio.resources_lookup_data(path, 0);
  return new TextDecoder().decode(data.get_data()?.buffer);
}

export class DbusBase {
  proxy!: Gio.DBusProxy;
  connected = false;
  ifaceXml = "";
  dbus_path = "";

  constructor(file_name: string, dbus_path: string) {
    let extensionObject = Extension.lookupByUUID("asusctl-gnome@asus-linux.org");
    const path = extensionObject?.path + "/resources/dbus/" + file_name;
    const [ok, data] = GLib.file_get_contents(path);
    if (!ok) {
      throw new Error("could not read interface file");
    }
    this.ifaceXml = imports.byteArray.toString(data);
    this.dbus_path = dbus_path;
  }

  async start() {
    //@ts-ignore
    log(`Starting ${this.dbus_path} dbus module`);
    try {
      log(this.ifaceXml);
      this.proxy = Gio.DBusProxy.makeProxyWrapper(this.ifaceXml)(
        Gio.DBus.system,
        "org.asuslinux.Daemon",
        this.dbus_path,
      );

      this.connected = true;
      //@ts-ignore
      log(`${this.dbus_path} client started successfully.`);
    } catch (e) {
      //@ts-ignore
      logError(`${this.xml_resource} dbus init failed!`, e);
    }
  }

  async stop() {
    //@ts-ignore
    log(`Stopping ${this.xml_resource} dbus module`);

    if (this.connected && this.proxy != undefined) {
      this.proxy.run_dispose();
      this.proxy = undefined;
      this.connected = false;
    }
  }

  isRunning(): boolean {
    return this.connected;
  }
}
