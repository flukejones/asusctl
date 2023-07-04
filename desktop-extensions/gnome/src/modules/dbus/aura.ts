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
    // TODO: interface or something to enforce requirement of "sync()" method
    public notifyAuraModeSubscribers: any[] = [];
    public notifyAuraPowerSubscribers: any[] = [];

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

    _parsePowerStates(data: any[]) {
        const power: AuraPowerDev = {
            tuf: [],
            x1866: [],
            x19b6: []
        };

        power.tuf = data[0].map((value: string) => {
            return AuraDevTuf[value as AuraDevTuf];
        });
        power.x1866 = data[1].map((value: string) => {
            return AuraDevRog1[value as AuraDevRog1];
        });
        power.x19b6 = data[2].map((value: string) => {
            return AuraDevRog2[value as AuraDevRog2];
        });

        return power;
    }

    public getLedPower() {
        if (this.isRunning()) {
            try {
                const data = this.dbus_proxy.LedPowerSync();
                this.leds_powered = this._parsePowerStates(data);
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

    public setLedMode(mode: AuraEffect) {
        if (this.isRunning()) {
            try {
                this.dbus_proxy.SetLedModeSync(mode);
            } catch (e) {
                //@ts-ignore
                log("Failed to fetch supported functionalities", e);
            }
        }
    }

    _parseAuraEffect(data: any[]) {
        const aura: AuraEffect = {
            mode: AuraModeNum[data[0] as AuraModeNum],
            zone: AuraZone[data[1] as AuraZone],
            colour1: {
                r: parseInt(data[2][0]),
                g: parseInt(data[2][1]),
                b: parseInt(data[2][2]),
            },
            colour2: {
                r: parseInt(data[3][0]),
                g: parseInt(data[3][1]),
                b: parseInt(data[3][2]),
            },
            speed: Speed[data[4] as Speed],
            direction: Direction[data[5] as Direction],
        };
        return aura;
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
                    const data = _data[0][key];
                    const aura: AuraEffect = this._parseAuraEffect(data);
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

            //@ts-ignore
            log("Current LED mode data:", this.aura_modes.get(this.current_aura_mode)?.speed);

            this.dbus_proxy.connectSignal(
                "NotifyLed",
                (proxy: any = null, name: string, data: any) => {
                    if (proxy) {
                        const aura: AuraEffect = this._parseAuraEffect(data[0]);
                        this.current_aura_mode = aura.mode;
                        this.aura_modes.set(aura.mode, aura);
                        //@ts-ignore
                        log("LED data has changed to ", aura.mode, aura.zone, aura.colour1.r, aura.speed, aura.direction);
                        this.notifyAuraModeSubscribers.forEach(sub => {
                            sub.sync();
                        });
                    }
                }
            );

            this.dbus_proxy.connectSignal(
                "NotifyPowerStates",
                (proxy: any = null, name: string, data: any) => {
                    if (proxy) {
                        const power: AuraPowerDev = this._parsePowerStates(data[0]);
                        this.leds_powered = power;
                        switch (this.device) {
                        case AuraDevice.Tuf:
                            //@ts-ignore
                            log("LED power has changed to ", this.leds_powered.tuf);
                            break;
                        case AuraDevice.X1854:
                        case AuraDevice.X1869:
                        case AuraDevice.X18c6:
                            //@ts-ignore
                            log("LED power has changed to ", this.leds_powered.x1866);
                            break;
                        case AuraDevice.X19b6:
                        case AuraDevice.X1a30:
                            //@ts-ignore
                            log("LED power has changed to ", this.leds_powered.x19b6);
                            break;
                        default:
                            break;
                        }
                        //@ts-ignore
                        log("LED power has changed to ", this.leds_powered.x19b6);
                        this.notifyAuraPowerSubscribers.forEach(sub => {
                            sub.sync();
                        });
                    }
                }
            );
        } catch (e) {
            //@ts-ignore
            log("Supported DBus initialization failed!", e);
        }
    }

    async stop() {
        await super.stop();
    }
}