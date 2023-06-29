declare const global: any, imports: any;
declare var asusctlGexInstance: any;
//@ts-ignore
const Me = imports.misc.extensionUtils.getCurrentExtension();

// REF: https://gjs.guide/extensions/development/creating.html

const { GObject, Gio } = imports.gi;
const ExtensionUtils = imports.misc.extensionUtils;

const { QuickToggle, SystemIndicator } = imports.ui.quickSettings;
const QuickSettingsMenu = imports.ui.main.panel.statusArea.quickSettings;

import { AnimeDbus } from './modules/anime_dbus';
import { Power } from './modules/power_dbus';
import { Supported } from './modules/supported_dbus';
import { Platform } from './modules/platform_dbus';

const QuickMiniLed = GObject.registerClass(
    class QuickMiniLed extends QuickToggle {
        _init() {
            super._init({
                title: 'MiniLED',
                iconName: 'selection-mode-symbolic',
                toggleMode: true,
                checked: asusctlGexInstance.dbus_platform.bios.mini_led_mode,
            });

            this.label = 'MiniLED';

            // Binding the toggle to a GSettings key
            this._settings = ExtensionUtils.getSettings();

            this.connectObject(
                'destroy', () => this._settings.run_dispose(),
                'clicked', () => this._toggleMode(),
                this);

            this._settings.bind('mini-led-enabled',
                this, 'checked',
                Gio.SettingsBindFlags.DEFAULT);

            this._sync();
        }

        _toggleMode() {
            asusctlGexInstance.dbus_platform.setMiniLedMode(this.checked);
            this._sync();
        }

        _sync() {
            const checked = asusctlGexInstance.dbus_platform.getMiniLedMode();
            if (this.checked !== checked)
                this.set({ checked });
            // this.set_property('checked', checked);
        }
    });

const IndicateMiniLed = GObject.registerClass(
    class IndicateMiniLed extends SystemIndicator {
        _init() {
            super._init();

            // Create the icon for the indicator
            this._indicator = this._addIndicator();
            this._indicator.icon_name = 'selection-mode-symbolic';

            // Showing the indicator when the feature is enabled
            this._settings = ExtensionUtils.getSettings();
            this._settings.bind('mini-led-enabled',
                this._indicator, 'visible',
                Gio.SettingsBindFlags.DEFAULT);

            // Create the toggle and associate it with the indicator, being sure to
            // destroy it along with the indicator
            this.quickSettingsItems.push(new QuickMiniLed());

            this.connect('destroy', () => {
                this.quickSettingsItems.forEach((item: { destroy: () => any; }) => item.destroy());
            });

            // Add the indicator to the panel and the toggle to the menu
            QuickSettingsMenu._indicators.add_child(this);
            QuickSettingsMenu._addItems(this.quickSettingsItems);
        }
    });


const QuickPanelOd = GObject.registerClass(
    class QuickPanelOd extends QuickToggle {
        _init() {
            super._init({
                title: 'Panel Overdrive',
                iconName: 'selection-mode-symbolic',
                toggleMode: true,
            });
            this.label = 'Panel Overdrive';
            this._settings = ExtensionUtils.getSettings();

            this.connectObject(
                'destroy', () => this._settings.run_dispose(),
                'clicked', () => this._toggleMode(),
                this);
            this._sync();
        }

        _toggleMode() {
            asusctlGexInstance.dbus_platform.setPanelOd(this.checked);
            this._sync();
        }

        _sync() {
            const checked = asusctlGexInstance.dbus_platform.getPanelOd();
            if (this.checked !== checked)
                this.set({ checked });
        }
    });

const IndicatePanelOd = GObject.registerClass(
    class IndicatePanelOd extends SystemIndicator {
        _init() {
            super._init();

            this.quickSettingsItems.push(new QuickPanelOd());
            // this.connect('destroy', () => {
            //     this.quickSettingsItems.forEach((item: { destroy: () => any; }) => item.destroy());
            // });
            // QuickSettingsMenu._indicators.add_child(this);
            QuickSettingsMenu._addItems(this.quickSettingsItems);
        }
    });

class Extension {
    private _indicateMiniLed: typeof IndicateMiniLed;
    private _indicatePanelOd: typeof IndicatePanelOd;
    private _dbus_power!: Power;
    dbus_platform!: Platform;
    private _dbus_anime!: AnimeDbus;
    dbus_supported!: Supported;

    constructor() {
        this._indicateMiniLed = null;
        this._indicatePanelOd = null;

        this.dbus_supported = new Supported();
        this.dbus_platform = new Platform();
        this._dbus_power = new Power();
        this._dbus_anime = new AnimeDbus();

        this.dbus_supported.start();
        this.dbus_platform.start();
        this._dbus_power.start();
        this._dbus_anime.start();
    }

    enable() {
        this._indicateMiniLed = new IndicateMiniLed();
        this._indicateMiniLed.checked = this.dbus_platform.bios.mini_led_mode;
        this._indicatePanelOd = new IndicatePanelOd();
    }

    disable() {
        this._indicateMiniLed.destroy();
        this._indicateMiniLed = null;
        this._indicatePanelOd.destroy();
        this._indicatePanelOd = null;

        this._dbus_power.stop();
        this.dbus_platform.stop();
        this._dbus_anime.stop();
        this.dbus_supported.stop();
    }
}

//@ts-ignore
function init() {
    asusctlGexInstance = new Extension();
    return new Extension();
}