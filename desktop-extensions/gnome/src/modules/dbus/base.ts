declare const imports: any;
//@ts-ignore
const Me = imports.misc.extensionUtils.getCurrentExtension();

import * as Resources from '../resources';

const { Gio } = imports.gi;

export class DbusBase {
    dbus_proxy: any = null; // type: Gio.DbusProxy
    connected: boolean = false;
    xml_resource: string = '';
    dbus_path: string = '';

    constructor(resource: string, dbus_path: string) {
        this.xml_resource = resource;
        this.dbus_path = dbus_path;
    }

    async start() {
        //@ts-ignore
        log(`Starting ${this.dbus_path} dbus module`);
        try {
            let xml = Resources.File.DBus(this.xml_resource);
            this.dbus_proxy = new Gio.DBusProxy.makeProxyWrapper(xml)(
                Gio.DBus.system,
                'org.asuslinux.Daemon',
                this.dbus_path,
            );

            this.connected = true;
            //@ts-ignore
            log(`${this.dbus_path} client started successfully.`);
        } catch (e) {
            //@ts-ignore
            log(`${this.xml_resource} dbus init failed!`, e);
        }
    }

    async stop() {
        //@ts-ignore
        log(`Stopping ${this.xml_resource} dbus module`);

        if (this.connected) {
            this.dbus_proxy.destroy();
            this.connected = false;
            this.dbus_proxy = null;
        }
    }

    isRunning(): boolean {
        return this.connected;
    }
}