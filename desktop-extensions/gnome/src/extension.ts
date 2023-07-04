// REF: https://gjs.guide/extensions/development/creating.html

import { AnimeDbus } from "./modules/dbus/animatrix";
import { Power } from "./modules/dbus/power";
import { Supported } from "./modules/dbus/supported";
import { Platform } from "./modules/dbus/platform";

import { QuickPanelOd } from "./modules/quick_toggles/panel_od";
import { IndicateMiniLed } from "./modules/indicators/mini_led";
import { QuickMiniLed } from "./modules/quick_toggles/mini_led";
import { SliderChargeLevel } from "./modules/sliders/charge";
import { QuickAnimePower } from "./modules/quick_toggles/anime_power";
import { FeatureMenuToggle } from "./modules/quick_menus/laptop_features";
import { AuraDbus } from "./modules/dbus/aura";
import { AuraMenuToggle } from "./modules/quick_menus/aura";

class Extension {
    private _indicateMiniLed: typeof IndicateMiniLed;
    private _quickMiniLed: typeof QuickMiniLed;
    private _quickPanelOd: typeof QuickPanelOd;
    private _quickAnimePower: typeof QuickAnimePower;
    private _featureMenuToggle: typeof FeatureMenuToggle;
    private _auraModeMenuToggle: typeof AuraMenuToggle;
    private _sliderCharge: typeof SliderChargeLevel;

    public dbus_supported: Supported = new Supported;
    public dbus_power: Power = new Power;
    public dbus_aura: AuraDbus = new AuraDbus;
    public dbus_anime: AnimeDbus = new AnimeDbus;
    public dbus_platform: Platform = new Platform;


    constructor() {
        this._indicateMiniLed = null;
        this._quickMiniLed = null;
        this._quickPanelOd = null;
        this._quickAnimePower = null;
        this._sliderCharge = null;

        this.dbus_supported.start();
        this.dbus_aura.start();
        this.dbus_platform.start();
        this.dbus_power.start();
        this.dbus_anime.start();
    }

    enable() {
        if (this._featureMenuToggle == null) {
            this._featureMenuToggle = new FeatureMenuToggle(this.dbus_supported, this.dbus_platform, this.dbus_anime);
        }
        if (this._auraModeMenuToggle == null) {
            this._auraModeMenuToggle = new AuraMenuToggle(this.dbus_aura);
        }
        if (this.dbus_supported.supported.rog_bios_ctrl.mini_led_mode) {
            // if (this._quickMiniLed == null) {
            //     this._quickMiniLed = new QuickMiniLed(this.dbus_platform);
            //     this.dbus_platform.notifyMiniLedSubscribers.push(this._quickMiniLed);
            // }
            if (this._indicateMiniLed == null) {
                this._indicateMiniLed = new IndicateMiniLed(this.dbus_platform);
            }
        }
        // if (this.dbus_supported.supported.rog_bios_ctrl.panel_overdrive) {
        //     if (this._quickPanelOd == null) {
        //         this._quickPanelOd = new QuickPanelOd(this.dbus_platform);
        //         this.dbus_platform.notifyPanelOdSubscribers.push(this._quickPanelOd);
        //     }
        // }
        // if (this.dbus_supported.supported.anime_ctrl) {
        //     if (this._quickAnimePower == null) {
        //         this._quickAnimePower = new QuickAnimePower(this._dbus_anime);
        //     }
        // }
        if (this.dbus_supported.supported.charge_ctrl.charge_level_set) {
            if (this._sliderCharge == null) {
                this._sliderCharge = new SliderChargeLevel(this.dbus_power);
            }
        }
    }

    disable() {
        if (this._indicateMiniLed != null) {
            this._indicateMiniLed.destroy();
            this._indicateMiniLed = null;
        }
        if (this._quickMiniLed != null) {
            this._quickMiniLed.destroy();
            this._quickMiniLed = null;
        }
        if (this._quickPanelOd != null) {
            this._quickPanelOd.destroy();
            this._quickPanelOd = null;
        }
        if (this._quickAnimePower != null) {
            this._quickAnimePower.destroy();
            this._quickAnimePower = null;
        }
        if (this._sliderCharge != null) {
            this._sliderCharge.destroy();
            this._sliderCharge = null;
        }

        this.dbus_power.stop();
        this.dbus_platform.stop();
        this.dbus_anime.stop();
        this.dbus_aura.stop();
        this.dbus_supported.stop();
    }
}

// eslint-disable-next-line @typescript-eslint/no-unused-vars
function init() {
    return new Extension();
}