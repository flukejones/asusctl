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

class Extension {
    private _indicateMiniLed: typeof IndicateMiniLed;
    private _quickMiniLed: typeof QuickMiniLed;
    private _quickPanelOd: typeof QuickPanelOd;
    private _quickAnimePower: typeof QuickAnimePower;
    private _sliderCharge: typeof SliderChargeLevel;

    public _dbus_power!: Power;
    public _dbus_anime!: AnimeDbus;
    public dbus_platform!: Platform;
    public dbus_supported!: Supported;

    constructor() {
        this._indicateMiniLed = null;
        this._quickMiniLed = null;
        this._quickPanelOd = null;
        this._quickAnimePower = null;
        this._sliderCharge = null;

        this.dbus_supported = new Supported();
        this.dbus_supported.start();

        this.dbus_platform = new Platform();
        this.dbus_platform.start();

        this._dbus_power = new Power();
        this._dbus_power.start();

        this._dbus_anime = new AnimeDbus();
        this._dbus_anime.start();
    }

    enable() {
        if (this.dbus_supported.supported.rog_bios_ctrl.mini_led_mode) {
            if (this._quickMiniLed == null) {
                this._quickMiniLed = new QuickMiniLed(this.dbus_platform);
            }
            if (this._indicateMiniLed == null) {
                this._indicateMiniLed = new IndicateMiniLed(this.dbus_platform);
            }
        }
        if (this.dbus_supported.supported.rog_bios_ctrl.panel_overdrive) {
            if (this._quickPanelOd == null) {
                this._quickPanelOd = new QuickPanelOd(this.dbus_platform);
            }
        }
        if (this.dbus_supported.supported.anime_ctrl) {
            if (this._quickAnimePower == null) {
                this._quickAnimePower = new QuickAnimePower(this._dbus_anime);
            }
        }
        if (this.dbus_supported.supported.charge_ctrl.charge_level_set) {
            if (this._sliderCharge == null) {
                this._sliderCharge = new SliderChargeLevel(this._dbus_power);
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

        this._dbus_power.stop();
        this.dbus_platform.stop();
        this._dbus_anime.stop();
        this.dbus_supported.stop();
    }
}

// eslint-disable-next-line @typescript-eslint/no-unused-vars
function init() {
    return new Extension();
}