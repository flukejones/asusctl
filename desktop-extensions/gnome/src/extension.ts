declare const imports: any;
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
import { addQuickSettingsItems } from "./modules/helpers";
import { MenuToggleAnimePower } from "./modules/menu_toggles/anime_power";
import { MenuTogglePanelOd } from "./modules/menu_toggles/panel_od";
import { MenuToggleMiniLed } from "./modules/menu_toggles/mini_led";


const { GObject } = imports.gi;

const ExtensionUtils = imports.misc.extensionUtils;
const Me = ExtensionUtils.getCurrentExtension();

const Main = imports.ui.main;
const PopupMenu = imports.ui.popupMenu;
const QuickSettings = imports.ui.quickSettings;

const FeatureMenuToggle = GObject.registerClass(
    class FeatureMenuToggle extends QuickSettings.QuickMenuToggle {
        private _dbus_supported: Supported;
        private _dbus_platform: Platform;
        private _dbus_power: Power;
        private _dbus_anime: AnimeDbus;

        public miniLed: typeof MenuToggleMiniLed;
        public panelOd: typeof MenuTogglePanelOd;
        public animePower: typeof MenuToggleAnimePower;

        constructor(dbus_supported: Supported, dbus_platform: Platform, dbus_power: Power, dbus_anime: AnimeDbus) {
            super({
                title: "Feature Name",
                iconName: "selection-mode-symbolic",
                toggleMode: true,
            });
            this._dbus_supported = dbus_supported;
            this._dbus_platform = dbus_platform;
            this._dbus_power = dbus_power;
            this._dbus_anime = dbus_anime;

            this.connectObject(
                "destroy", () => this._settings.run_dispose(),
                "clicked", () => log("TODO: change chosen primary thing"),
                this);

            this.menu.setHeader("selection-mode-symbolic", "Laptop features");

            this._itemsSection = new PopupMenu.PopupMenuSection();

            if (this._dbus_supported.supported.rog_bios_ctrl.mini_led_mode) {
                if (this.miniLed == null) {
                    this.miniLed = new MenuToggleMiniLed(this._dbus_platform);
                    this._dbus_platform.notifyMiniLedSubscribers.push(this.miniLed);
                    this._itemsSection.addMenuItem(this.miniLed);
                }
            }
            if (this._dbus_supported.supported.rog_bios_ctrl.panel_overdrive) {
                if (this.panelOd == null) {
                    this.panelOd = new MenuTogglePanelOd(this._dbus_platform);
                    this._dbus_platform.notifyPanelOdSubscribers.push(this.panelOd);
                    this._itemsSection.addMenuItem(this.panelOd);
                }
            }
            if (this._dbus_supported.supported.anime_ctrl) {
                if (this.animePower == null) {
                    this.animePower = new MenuToggleAnimePower(this._dbus_anime);
                    this._dbus_anime.notifyAnimeStateSubscribers.push(this.animePower);
                    this._itemsSection.addMenuItem(this.animePower);
                }
            }

            this.menu.addMenuItem(this._itemsSection);

            // Add an entry-point for more settings
            this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());
            const settingsItem = this.menu.addAction("More Settings",
                () => ExtensionUtils.openPrefs());

            // Ensure the settings are unavailable when the screen is locked
            settingsItem.visible = Main.sessionMode.allowSettings;
            this.menu._settingsActions[Me.uuid] = settingsItem;

            addQuickSettingsItems([this]);
        }
    });


class Extension {
    private _indicateMiniLed: typeof IndicateMiniLed;
    private _quickMiniLed: typeof QuickMiniLed;
    private _quickPanelOd: typeof QuickPanelOd;
    private _quickAnimePower: typeof QuickAnimePower;
    private _sliderCharge: typeof SliderChargeLevel;

    public dbus_supported: Supported = new Supported;
    public dbus_power: Power = new Power;
    public dbus_anime: AnimeDbus = new AnimeDbus;
    public dbus_platform: Platform = new Platform;


    constructor() {
        this._indicateMiniLed = null;
        this._quickMiniLed = null;
        this._quickPanelOd = null;
        this._quickAnimePower = null;
        this._sliderCharge = null;

        this.dbus_supported.start();
        this.dbus_platform.start();
        this.dbus_power.start();
        this.dbus_anime.start();
    }

    enable() {
        new FeatureMenuToggle(this.dbus_supported, this.dbus_platform, this.dbus_power, this.dbus_anime);
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
        this.dbus_supported.stop();
    }
}

// eslint-disable-next-line @typescript-eslint/no-unused-vars
function init() {
    return new Extension();
}