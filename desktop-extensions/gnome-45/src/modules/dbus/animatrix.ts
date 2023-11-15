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
        off_when_unplugged: false,
        off_when_suspended: false,
        off_when_lid_closed: false,
    };

    // TODO: interface or something to enforce requirement of "sync()" method
    public notifyAnimeStateSubscribers: any[] = [];

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

    public setPowersaveAnim(state: boolean | null) {
        if (this.isRunning()) {
            try {
                // if null, toggle the current state
                state = (state == null ? !this.deviceState.builtin_anims_enabled : state);

                if (this.deviceState.builtin_anims_enabled !== state) {
                    this.deviceState.builtin_anims_enabled = state;
                }
                return this.dbus_proxy.SetEnableBuiltinsSync(state);
            } catch (e) {
                //@ts-ignore
                log("AniMe DBus set builtins failed!", e);
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
            this.deviceState.builtin_anims.shutdown = AnimShutdown[data[3][3] as AnimShutdown];
            this.deviceState.off_when_unplugged = data[4];
            this.deviceState.off_when_suspended = data[5];
            this.deviceState.off_when_lid_closed = data[6];
        }
    }

    public getDeviceState() {
        if (this.isRunning()) {
            try {
                // janky shit going on with DeviceStateSync
                this._parseData(this.dbus_proxy.DeviceStateSync());
                //@ts-ignore
                log("Anime Matrix: display_enabled: " + this.deviceState.display_enabled);
                //@ts-ignore
                log("Anime Matrix: display_brightness: " + this.deviceState.display_brightness);
                //@ts-ignore
                log("Anime Matrix: builtin_anims_enabled: " + this.deviceState.builtin_anims_enabled);
                //@ts-ignore
                log("Anime Matrix: builtin_anims: " + this.deviceState.builtin_anims);
                //@ts-ignore
                log("Anime Matrix: off_when_unplugged: " + this.deviceState.off_when_unplugged);
                //@ts-ignore
                log("Anime Matrix: off_when_suspended: " + this.deviceState.off_when_suspended);
                //@ts-ignore
                log("Anime Matrix: off_when_lid_closed: " + this.deviceState.off_when_lid_closed);
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
                    this.notifyAnimeStateSubscribers.forEach(sub => {
                        sub.sync();
                    });
                }
            }
        );
    }

    async stop() {
        await super.stop();
    }
}