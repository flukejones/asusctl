import { AuraDevRog1, AuraDevRog2, AuraDevTuf, AuraDevice, AuraEffect, AuraModeNum, AuraPowerDev, AuraZone, Direction, Speed } from "../../bindings/aura";
import { DbusBase } from "./base";

export class AuraDbus extends DbusBase {
    public device: AuraDevice = AuraDevice.Unknown;
    public current_aura_mode: AuraModeNum = AuraModeNum.Static;
    public aura_modes: Map<AuraModeNum, AuraEffect> = new Map;
    public leds_powered: AuraPowerDev = {
        tuf: [],
        x1866: [],
        x19b6: []
    };

    constructor() {
        super("org-asuslinux-aura-4", "/org/asuslinux/Aura");
    }

    public getDevice() {
        if (this.isRunning()) {
            try {
                this.device = AuraDevice[this.dbus_proxy.DeviceTypeSync() as AuraDevice];
                //@ts-ignore
                log("LED device: " + this.device);
            } catch (e) {
                //@ts-ignore
                log("Failed to fetch supported functionalities", e);
            }
        }
    }

    public getLedPower() {
        if (this.isRunning()) {
            try {
                const data = this.dbus_proxy.LedPowerSync();
                this.leds_powered.tuf = data[0].map((value: string) => {
                    return AuraDevTuf[value as AuraDevTuf];
                });
                this.leds_powered.x1866 = data[1].map((value: string) => {
                    return AuraDevRog1[value as AuraDevRog1];
                });
                this.leds_powered.x19b6 = data[2].map((value: string) => {
                    return AuraDevRog2[value as AuraDevRog2];
                });
                //@ts-ignore
                log("LED power tuf: " + this.leds_powered.tuf);
                //@ts-ignore
                log("LED power x1866: " + this.leds_powered.x1866);
                //@ts-ignore
                log("LED power x19b6: " + this.leds_powered.x19b6);
            } catch (e) {
                //@ts-ignore
                log("Failed to fetch supported functionalities", e);
            }
        }
    }

    public getLedMode() {
        if (this.isRunning()) {
            try {
                this.current_aura_mode = AuraModeNum[this.dbus_proxy.LedModeSync() as AuraModeNum];
                //@ts-ignore
                log("Current LED mode:", this.current_aura_mode);
            } catch (e) {
                //@ts-ignore
                log("Failed to fetch supported functionalities", e);
            }
        }
    }

    // Return a list of the available modes, and the current settings for each
    public getLedModes() {
        // {'Breathe': ('Breathe', 'None', (166, 0, 0), (0, 0, 0), 'Med', 'Right'),
        // 'Comet': ('Comet', 'None', (166, 0, 0), (0, 0, 0), 'Med', 'Right'),
        // 'Static': ('Static', 'None', (78, 0, 0), (0, 0, 0), 'Med', 'Right'),
        // 'Strobe': ('Strobe', 'None', (166, 0, 0), (0, 0, 0), 'Med', 'Right')}
        if (this.isRunning()) {
            try {
                const _data = this.dbus_proxy.LedModesSync();
                for (const key in _data[0]) {
                    const value = _data[0][key];
                    const aura: AuraEffect = {
                        mode: AuraModeNum[value[0] as AuraModeNum],
                        zone: AuraZone[value[1] as AuraZone],
                        colour1: {
                            r: parseInt(value[2][0]),
                            g: parseInt(value[2][1]),
                            b: parseInt(value[2][2]),
                        },
                        colour2: {
                            r: parseInt(value[3][0]),
                            g: parseInt(value[3][1]),
                            b: parseInt(value[3][2]),
                        },
                        speed: Speed[value[4] as Speed],
                        direction: Direction[value[5] as Direction],
                    };
                    this.aura_modes.set(AuraModeNum[key as AuraModeNum], aura);
                }

                for (const [key, value] of this.aura_modes) {
                    //@ts-ignore
                    log(key, value.zone, value.colour1.r, value.speed, value.direction);
                }

            } catch (e) {
                //@ts-ignore
                log("Failed to fetch supported functionalities", e);
            }
        }
    }

    async start() {
        try {
            await super.start();
            this.getDevice();
            this.getLedPower();
            this.getLedMode();
            this.getLedModes();
        } catch (e) {
            //@ts-ignore
            log("Supported DBus initialization failed!", e);
        }
    }

    async stop() {
        await super.stop();
    }
}