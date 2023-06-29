declare const global: any, imports: any;
declare var asusctlGexInstance: any;
//@ts-ignore
const ThisModule = imports.misc.extensionUtils.getCurrentExtension();

import * as Resources from './resources';

const { Gio } = imports.gi;

import { DbusBase } from '../modules/dbus';

export class AnimeDbus extends DbusBase {
    state: boolean = true;
    brightness: number = 255;

    constructor() {
        super('org-asuslinux-anime-4', '/org/asuslinux/Anime');
    }

    public setOnOffState(state: boolean | null) {
        if (this.isRunning()) {
            try {
                // if null, toggle the current state
                state = (state == null ? !this.state : state);

                if (this.state !== state) {
                    this.state = state;
                }
                //@ts-ignore
                log(`Setting AniMe Power to ${state}`);
                return this.dbus_proxy.SetOnOffSync(state);
            } catch (e) {
                //@ts-ignore
                log(`AniMe DBus set power failed!`, e);
            }
        }
    }

    public setBrightness(brightness: number) {
        if (this.isRunning()) {
            try {
                if (this.brightness !== brightness) {
                    this.brightness = brightness;
                }
                //@ts-ignore
                log(`Setting AniMe Brightness to ${brightness}`);
                return this.dbus_proxy.SetBrightnessSync(brightness);
                // Panel.Actions.spawnCommandLine(`asusctl anime leds -b ${brightness}`);
            } catch (e) {
                //@ts-ignore
                log(`AniMe DBus set brightness failed!`, e);
            }
        }
    }

    async start() {
        //@ts-ignore
        log(`Starting AniMe DBus client...`);

        try {
            // creating the proxy
            let xml = Resources.File.DBus('org-asuslinux-anime-4')
            this.dbus_proxy = new Gio.DBusProxy.makeProxyWrapper(xml)(
                Gio.DBus.system,
                'org.asuslinux.Daemon',
                '/org/asuslinux/Anime'
            );

            this.connected = true;

            // currently there is no DBUS method because this can't be read from
            // hardware (as to @fluke).
            // https://gitlab.com/asus-linux/asusctl/-/issues/138
            /*
              this.asusLinuxProxy.connectSignal(
                  "NotifyCharge",
                  (proxy: any = null, name: string, data: string) => {
                      if (proxy) {
                          Log.info(`AniMe Power State has changed to ${data}% (${name}).`);
                      }
                  }
              );
            */
        } catch (e) {
            //@ts-ignore
            log(`AniMe DBus initialization failed!`, e);
        }
    }

    async stop() {
        await super.stop();
    }
}