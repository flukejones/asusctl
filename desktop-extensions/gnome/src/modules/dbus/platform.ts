declare const imports: any;
//@ts-ignore
const Me = imports.misc.extensionUtils.getCurrentExtension();

import * as bios from '../../bindings/platform';
import { DbusBase } from './base';

// TODO: add callbacks for notifications
export class Platform extends DbusBase {
    bios: bios.RogBiosSupportedFunctions = {
        post_sound: false,
        gpu_mux: false,
        panel_overdrive: false,
        dgpu_disable: false,
        egpu_enable: false,
        mini_led_mode: false
    }

    constructor() {
        super('org-asuslinux-platform-4', '/org/asuslinux/Platform');
    }

    public getPostBootSound() {
        if (this.isRunning()) {
            try {
                this.bios.post_sound = this.dbus_proxy.PostBootSoundSync() == 'true' ? true : false;
            } catch (e) {
                //@ts-ignore
                log(`Failed to get POST Boot Sound state!`, e);
            }
        }
        return this.bios.post_sound;
    }

    public setPostBootSound(state: boolean) {
        if (this.isRunning()) {
            try {
                if (state !== this.bios.post_sound) {
                    this.bios.post_sound = state;
                }
                return this.dbus_proxy.SetPostBootSoundSync(state);
            } catch (e) {
                //@ts-ignore
                log(`Platform DBus set Post Boot Sound failed!`, e);
            }
        }
    }

    public getGpuMuxMode() {
        if (this.isRunning()) {
            try {
                this.bios.gpu_mux = this.dbus_proxy.GpuMuxModeSync() == 'true' ? true : false;
            } catch (e) {
                //@ts-ignore
                log(`Failed to get MUX state!`, e);
            }
        }
        return this.bios.gpu_mux;
    }

    public setGpuMuxMode(state: boolean) {
        if (this.isRunning()) {
            try {
                if (!state !== this.bios.gpu_mux) {
                    this.bios.gpu_mux = !state;
                }
                return this.dbus_proxy.SetGpuMuxModeSync(!state);
            } catch (e) {
                //@ts-ignore
                log(`Switching the MUX failed!`, e);
            }
        }
    }

    public getPanelOd() {
        if (this.isRunning()) {
            try {
                this.bios.panel_overdrive = this.dbus_proxy.PanelOdSync() == 'true' ? true : false;
            } catch (e) {
                //@ts-ignore
                log(`Failed to get Overdrive state!`, e);
            }
        }
        return this.bios.panel_overdrive;
    }

    public setPanelOd(state: boolean) {
        if (this.isRunning()) {
            try {
                if (state !== this.bios.panel_overdrive) {
                    this.bios.panel_overdrive = state;
                }
                return this.dbus_proxy.SetPanelOdSync(state);
            } catch (e) {
                //@ts-ignore
                log(`Overdrive DBus set overdrive state failed!`, e);
            }
        }
    }

    public getMiniLedMode() {
        if (this.isRunning()) {
            try {
                this.bios.mini_led_mode = this.dbus_proxy.MiniLedModeSync() == 'true' ? true : false;
            } catch (e) {
                //@ts-ignore
                log(`Failed to get Overdrive state!`, e);
            }
        }
        return this.bios.mini_led_mode;
    }

    public setMiniLedMode(state: boolean) {
        if (this.isRunning()) {
            try {
                if (state !== this.bios.mini_led_mode) {
                    this.bios.mini_led_mode = state;
                }
                return this.dbus_proxy.SetMiniLedModeSync(state);
            } catch (e) {
                //@ts-ignore
                log(`setMiniLedMode failed!`, e);
            }
        }
    }

    async start() {
        try {
            await super.start();

            this.getPostBootSound();
            this.dbus_proxy.connectSignal(
                "NotifyPostBootSound",
                (proxy: any = null, _name: string, data: boolean) => {
                    if (proxy) {
                        //@ts-ignore
                        log(`PostBootSound changed to ${data}`);
                    }
                }
            );

            this.getPanelOd();
            this.dbus_proxy.connectSignal(
                "NotifyPanelOd",
                (proxy: any = null, _name: string, data: boolean) => {
                    if (proxy) {
                        //@ts-ignore
                        log(`NotifyPanelOd has changed to ${data}.`);
                    }
                }
            );

            this.getMiniLedMode();
            this.dbus_proxy.connectSignal(
                "NotifyMiniLedMode",
                (proxy: any = null, _name: string, data: boolean) => {
                    if (proxy) {
                        //@ts-ignore
                        log(`MiniLedMode has changed to ${data}.`);
                    }
                }
            );

            this.getGpuMuxMode();
            this.dbus_proxy.connectSignal(
                "NotifyGpuMuxMode",
                (proxy: any = null, _name: string, data: boolean) => {
                    if (proxy) {
                        //@ts-ignore
                        log(`MUX has changed to ${data}.`);
                    }
                }
            );

        } catch (e) {
            //@ts-ignore
            log(`Platform DBus init failed!`, e);
        }
    }

    async stop() {
        await super.stop();
        this.bios.post_sound = false;
        this.bios.panel_overdrive = false;
        this.bios.mini_led_mode = false;
        this.bios.gpu_mux = false;
    }
}