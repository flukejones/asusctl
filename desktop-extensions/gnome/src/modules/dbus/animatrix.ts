declare const imports: any;
//@ts-ignore
const Me = imports.misc.extensionUtils.getCurrentExtension();

import { DbusBase } from './base';
import { DeviceState, AnimBooting, Brightness, AnimAwake, AnimSleeping, AnimShutdown } from '../../bindings/anime';

export class AnimeDbus extends DbusBase {
    deviceState: DeviceState = {
        display_enabled: false,
        display_brightness: Brightness.Med,
        builtin_anims_enabled: false,
        builtin_anims: {
            boot: AnimBooting.GlitchConstruction,
            awake: AnimAwake.BinaryBannerScroll,
            sleep: AnimSleeping.BannerSwipe,
            shutdown: AnimShutdown.GlitchOut
        },
    };

    constructor() {
        super('org-asuslinux-anime-4', '/org/asuslinux/Anime');
    }

    public setEnableDisplay(state: boolean | null) {
        if (this.isRunning()) {
            try {
                // if null, toggle the current state
                state = (state == null ? !this.deviceState.display_enabled : state);

                if (this.deviceState.display_enabled !== state) {
                    this.deviceState.display_enabled = state;
                }
                return this.dbus_proxy.SetEnableDisplaySync(state);
            } catch (e) {
                //@ts-ignore
                log(`AniMe DBus set power failed!`, e);
            }
        }
    }

    public setBrightness(brightness: Brightness) {
        if (this.isRunning()) {
            try {
                if (this.deviceState.display_brightness !== brightness) {
                    this.deviceState.display_brightness = brightness;
                }
                return this.dbus_proxy.SetBrightnessSync(brightness);
            } catch (e) {
                //@ts-ignore
                log(`AniMe DBus set brightness failed!`, e);
            }
        }
    }

    public getDeviceState() {
        if (this.isRunning()) {
            try {
                let _data = this.dbus_proxy.DeviceStateSync();
                if (_data.length > 0) {
                    this.deviceState.display_enabled = _data[0];
                    this.deviceState.display_brightness = Brightness[_data[1] as Brightness];
                    this.deviceState.builtin_anims_enabled = _data[2];
                    this.deviceState.builtin_anims.boot = AnimBooting[_data[3][0] as AnimBooting];
                    this.deviceState.builtin_anims.awake = AnimAwake[_data[3][1] as AnimAwake];
                    this.deviceState.builtin_anims.sleep = AnimSleeping[_data[3][2] as AnimSleeping];
                    this.deviceState.builtin_anims.shutdown = AnimShutdown[_data[3][2] as AnimShutdown];
                    // this._parseDeviceStateString(_data);
                }
            } catch (e) {
                //@ts-ignore
                log(`Failed to fetch DeviceState!`, e);
            }
        }
        return this.deviceState;
    }

    _parseDeviceStateString(input: String) {
        let valueString: string = '';

        for (const [_key, value] of Object.entries(input)) {
            //@ts-ignore
            valueString = value.toString();

            switch (parseInt(_key)) {
                case 0:
                    this.deviceState.display_enabled = (valueString == 'true' ? true : false);
                    break;
                case 1:
                    this.deviceState.display_brightness = Brightness[valueString as Brightness];
                    break;
                case 2:
                    this.deviceState.builtin_anims_enabled = (valueString == 'true' ? true : false);
                    break;
                case 3:
                    let anims = valueString.split(',');
                    this.deviceState.builtin_anims.boot = AnimBooting[anims[0] as AnimBooting];
                    this.deviceState.builtin_anims.awake = AnimAwake[anims[1] as AnimAwake];
                    this.deviceState.builtin_anims.sleep = AnimSleeping[anims[2] as AnimSleeping];
                    this.deviceState.builtin_anims.shutdown = AnimShutdown[anims[3] as AnimShutdown];
                    break;
            }
        }
    }


    async start() {
        await super.start();
        this.getDeviceState();

        this.dbus_proxy.connectSignal(
            "NotifyDeviceState",
            (proxy: any = null, name: string, data: string) => {
                if (proxy) {
                    this._parseDeviceStateString(data);
                    //@ts-ignore
                    log(`NotifyDeviceState has changed to ${data}`);
                }
            }
        );
    }

    async stop() {
        await super.stop();
    }
}