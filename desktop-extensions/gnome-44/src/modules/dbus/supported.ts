import { SupportedFunctions, AdvancedAura } from "../../bindings/platform";
import { AuraDevice, AuraModeNum, AuraZone, PowerZones } from "../../bindings/aura";
import { DbusBase } from "./base";

export class Supported extends DbusBase {
    // False,
    // (True,),
    // (True, True),
    //   ('X19b6',
    //   True,
    //   ['Static',
    //   'Breathe',
    //   'Strobe',
    //   'Rainbow',
    //   'Star',
    //   'Rain',
    //   'Highlight',
    //   'Laser',
    //   'Ripple',
    //   'Pulse',
    //   'Comet',
    //   'Flash'],
    //   [],
    //   'PerKey',
    //   ['Keyboard', 'Lightbar', 'Logo', 'RearGlow']),
    // (False, True, True, True, False, True)

    supported: SupportedFunctions = {
        anime_ctrl: false,
        charge_ctrl: {
            charge_level_set: false
        },
        platform_profile: {
            platform_profile: false,
            fan_curves: false
        },
        keyboard_led: {
            dev_id: AuraDevice.Unknown,
            brightness: false,
            basic_modes: [],
            basic_zones: [],
            advanced_type: AdvancedAura.None
        },
        rog_bios_ctrl: {
            post_sound: false,
            gpu_mux: false,
            panel_overdrive: false,
            dgpu_disable: false,
            egpu_enable: false,
            mini_led_mode: false
        }
    };

    constructor() {
        super("org-asuslinux-supported-4", "/org/asuslinux/Supported");
    }

    public getSupported() {
        if (this.isRunning()) {
            try {
                const _data = this.dbus_proxy.SupportedFunctionsSync();
                this.supported.anime_ctrl = _data[0];
                this.supported.charge_ctrl.charge_level_set = _data[1];
                this.supported.platform_profile.platform_profile = _data[2][0];
                this.supported.platform_profile.fan_curves = _data[2][1];
                this.supported.keyboard_led.dev_id = AuraDevice[_data[3][0] as AuraDevice];
                this.supported.keyboard_led.brightness = _data[3][1];

                this.supported.keyboard_led.basic_modes = _data[3][2].map(function (value: string) {
                    return AuraModeNum[value as AuraModeNum];
                });
                this.supported.keyboard_led.basic_zones = _data[3][3].map(function (value: string) {
                    return AuraZone[value as AuraZone];
                });
                this.supported.keyboard_led.advanced_type = AdvancedAura[_data[3][4] as AdvancedAura];
                this.supported.keyboard_led.power_zones = _data[3][5].map(function (value: string) {
                    return PowerZones[value as PowerZones];
                });

                this.supported.rog_bios_ctrl.post_sound = _data[4][0];
                this.supported.rog_bios_ctrl.gpu_mux = _data[4][1];
                this.supported.rog_bios_ctrl.panel_overdrive = _data[4][2];
                this.supported.rog_bios_ctrl.dgpu_disable = _data[4][3];
                this.supported.rog_bios_ctrl.egpu_enable = _data[4][4];
                this.supported.rog_bios_ctrl.mini_led_mode = _data[4][5];
            } catch (e) {
                //@ts-ignore
                log("Failed to fetch supported functionalities", e);
            }
        }
    }

    async start() {
        try {
            await super.start();
            this.getSupported();
        } catch (e) {
            //@ts-ignore
            log("Supported DBus initialization failed!", e);
        }
    }

    async stop() {
        await super.stop();
    }
}