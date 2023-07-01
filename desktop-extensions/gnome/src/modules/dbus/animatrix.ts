import { DbusBase } from "./base";
import { DeviceState, AnimBooting, Brightness, AnimAwake, AnimSleeping, AnimShutdown } from "../../bindings/anime";

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
        super("org-asuslinux-anime-4", "/org/asuslinux/Anime");
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
                log("AniMe DBus set power failed!", e);
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
                log("AniMe DBus set brightness failed!", e);
            }
        }
    }

    _parseData(data: any) {
        if (data.length > 0) {
            this.deviceState.display_enabled = data[0];
            this.deviceState.display_brightness = Brightness[data[1] as Brightness];
            this.deviceState.builtin_anims_enabled = data[2];
            this.deviceState.builtin_anims.boot = AnimBooting[data[3][0] as AnimBooting];
            this.deviceState.builtin_anims.awake = AnimAwake[data[3][1] as AnimAwake];
            this.deviceState.builtin_anims.sleep = AnimSleeping[data[3][2] as AnimSleeping];
            this.deviceState.builtin_anims.shutdown = AnimShutdown[data[3][2] as AnimShutdown];
        }
    }

    public getDeviceState() {
        if (this.isRunning()) {
            try {
                // janky shit going on with DeviceStateSync
                this._parseData(this.dbus_proxy.DeviceStateSync());
            } catch (e) {
                //@ts-ignore
                log("Failed to fetch DeviceState!", e);
            }
        }
        return this.deviceState;
    }

    async start() {
        await super.start();
        this.getDeviceState();

        this.dbus_proxy.connectSignal(
            "NotifyDeviceState",
            // eslint-disable-next-line @typescript-eslint/no-explicit-any
            (proxy: any = null, name: string, data: string) => {
                if (proxy) {
                    // idiot xml parsing mneans the get is not nested while this is
                    this._parseData(data[0]);
                }
            }
        );
    }

    async stop() {
        await super.stop();
    }
}