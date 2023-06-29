declare const global: any, imports: any;
declare var asusctlGexInstance: any;
//@ts-ignore
const Me = imports.misc.extensionUtils.getCurrentExtension();

import * as bios from '../bindings/platform';
import { DbusBase } from '../modules/dbus';

export class Platform extends DbusBase {
    bios: bios.RogBiosSupportedFunctions = asusctlGexInstance.supported.connector.supported;

    constructor() {
        super('org-asuslinux-platform-4', '/org/asuslinux/Platform');
    }

    public getPostBootSound() {
        if (this.isRunning()) {
            try {
                let currentState = this.dbus_proxy.PostBootSoundSync();
                return parseInt(currentState) == 1 ? true : false;
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

    public getMUX() {
        if (this.isRunning()) {
            try {
                let currentState = this.dbus_proxy.GpuMuxModeSync();
                return parseInt(currentState) == 0 ? true : false;
            } catch (e) {
                //@ts-ignore
                log(`Failed to get MUX state!`, e);
            }
        }
        return this.bios.post_sound;
    }

    public setMUX(state: boolean) {
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

    public getOverdrive() {
        if (this.isRunning()) {
            try {
                let currentState = this.dbus_proxy.PanelOverdriveSync();
                return parseInt(currentState) == 1 ? true : false;
            } catch (e) {
                //@ts-ignore
                log(`Failed to get Overdrive state!`, e);
            }
        }
        return this.bios.panel_overdrive;
    }

    public setOverdrive(state: boolean) {
        if (this.isRunning()) {
            try {
                if (state !== this.bios.panel_overdrive) {
                    this.bios.panel_overdrive = state;
                }
                return this.dbus_proxy.SetPanelOverdriveSync(state);
            } catch (e) {
                //@ts-ignore
                log(`Overdrive DBus set overdrive state failed!`, e);
            }
        }
    }

    async start() {
        try {
            await super.start();

            if (asusctlGexInstance.supported.connector.supportedAttributes.bios_toggleSound) {
                this.bios.post_sound = this.getPostBootSound();
                this.dbus_proxy.connectSignal(
                    "NotifyPostBootSound",
                    (proxy: any = null, _name: string, data: boolean) => {
                        if (proxy) {
                            //@ts-ignore
                            log(`PostBootSound changed to ${data}`);
                            asusctlGexInstance.Platform.switchPostBootSound.setToggleState(this.bios.post_sound);
                        }
                    }
                );
            }

            if (asusctlGexInstance.supported.connector.supportedAttributes.bios_overdrive) {
                this.bios.panel_overdrive = this.getOverdrive();
                this.dbus_proxy.connectSignal(
                    "NotifyPanelOverdrive",
                    (proxy: any = null, _name: string, data: boolean) => {
                        if (proxy) {
                            //@ts-ignore
                            log(`Overdrive has changed to ${data}.`);
                            asusctlGexInstance.Platform.overdriveSwitch.setToggleState(this.bios.panel_overdrive);
                        }
                    }
                );
            }

            if (asusctlGexInstance.supported.connector.supportedAttributes.bios_toggleMUX) {
                this.bios.gpu_mux = this.getMUX();
                this.dbus_proxy.connectSignal(
                    "NotifyGpuMuxMode",
                    (proxy: any = null, _name: string, data: boolean) => {
                        if (proxy) {
                            //@ts-ignore
                            log(`MUX has changed to ${data}.`);
                            asusctlGexInstance.Platform.switchMUX.setToggleState(this.bios.gpu_mux);

                            // Panel.Actions.notify(
                            //     'ASUS Notebook Control',
                            //     `MUX Mode has chnged. Please reboot to apply the changes.`,
                            //     'scalable/reboot.svg',
                            //     'reboot'
                            // );
                        }
                    }
                );
            }
        } catch (e) {
            //@ts-ignore
            log(`Overdrive DBus init failed!`, e);
        }
    }

    async stop() {
        await super.stop();
        this.bios.post_sound = false;
        this.bios.panel_overdrive = false;
    }
}