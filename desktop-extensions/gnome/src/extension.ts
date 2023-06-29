declare const global: any, imports: any;
//@ts-ignore
const Me = imports.misc.extensionUtils.getCurrentExtension();

// const { GpuMode } = imports.bindings.platform;
// REF: https://gjs.guide/extensions/development/creating.html

const { GObject, Gio } = imports.gi;
const ExtensionUtils = imports.misc.extensionUtils;

const QuickSettings = imports.ui.quickSettings;
// This is the live instance of the Quick Settings menu
const QuickSettingsMenu = imports.ui.main.panel.statusArea.quickSettings;
//@ts-ignore
const ThisModule = imports.misc.extensionUtils.getCurrentExtension();

// const systemConnection = Gio.DBus.system;
// const TestProxy = Gio.DBusProxy.makeProxyWrapper(interfaceXml);

import * as Platform from './bindings/platform';
import { ChargingLimit } from './modules/charge_dbus';
import { Supported } from './modules/supported_dbus';

const QuickMiniLed = GObject.registerClass(
    class QuickMiniLed extends QuickSettings.QuickToggle {
        _init() {
            super._init({
                title: 'MiniLED',
                iconName: 'selection-mode-symbolic',
                toggleMode: true,
            });

            this.label = 'MiniLED';

            // Binding the toggle to a GSettings key
            this._settings = ExtensionUtils.getSettings();

            this._settings.bind('mini-led-enabled',
                this, 'checked',
                Gio.SettingsBindFlags.DEFAULT);
        }
    });

const IndicateMiniLed = GObject.registerClass(
    class IndicateMiniLed extends QuickSettings.SystemIndicator {
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
    class QuickPanelOd extends QuickSettings.QuickToggle {
        _init() {
            super._init({
                title: 'Panel Overdrive',
                iconName: 'selection-mode-symbolic',
                toggleMode: true,
            });
            this.label = 'Panel Overdrive';
            this._settings = ExtensionUtils.getSettings();
            this._settings.bind('panel-od-enabled',
                this, 'checked',
                Gio.SettingsBindFlags.DEFAULT);
        }
    });

const IndicatePanelOd = GObject.registerClass(
    class IndicatePanelOd extends QuickSettings.SystemIndicator {
        _init() {
            super._init();
            this._indicator = this._addIndicator();
            this._indicator.icon_name = 'selection-mode-symbolic';
            this._settings = ExtensionUtils.getSettings();
            this._settings.bind('panel-od-enabled',
                this._indicator, 'visible',
                Gio.SettingsBindFlags.DEFAULT);
            this.quickSettingsItems.push(new QuickPanelOd());
            this.connect('destroy', () => {
                this.quickSettingsItems.forEach((item: { destroy: () => any; }) => item.destroy());
            });
            QuickSettingsMenu._indicators.add_child(this);
            QuickSettingsMenu._addItems(this.quickSettingsItems);
        }
    });


function onNameAppeared(_connection: any, name: any, name_owner: any) {
    //@ts-ignore
    log(`The well-known name ${name} has been owned by ${name_owner}`);
}

// Likewise, this will be invoked when the process that owned the name releases
// the name.
function onNameVanished(_connection: any, name: any) {
    //@ts-ignore
    log(`The name owner of ${name} has vanished`);
}

const busWatchId = Gio.bus_watch_name(
    Gio.BusType.SESSION,
    'guide.gjs.Test',
    Gio.BusNameWatcherFlags.NONE,
    onNameAppeared,
    onNameVanished
);

Gio.bus_unwatch_name(busWatchId);

class Extension {
    //@ts-ignore
    private _naff: Platform.GpuMode;
    private _indicateMiniLed: typeof IndicateMiniLed;
    private _indicatePanelOd: typeof IndicatePanelOd;
    private _dbus_charge!: ChargingLimit;
    private _dbus_supported!: Supported;

    constructor() {
        this._indicateMiniLed = null;
        this._indicatePanelOd = null;
        this._naff = Platform.GpuMode.Discrete;
    }

    enable() {
        this._indicateMiniLed = new IndicateMiniLed();
        this._indicatePanelOd = new IndicatePanelOd();
        this._dbus_charge = new ChargingLimit();
        this._dbus_charge.start().then(() => {
            //@ts-ignore
            log(`DOOOOOM!, charge limit =`, this._dbus_charge.lastState);
        });
        this._dbus_supported = new Supported();
        this._dbus_supported.start().then(() => {
            //@ts-ignore
            log(`DOOOOOM!, supported =`, this._dbus_supported.supported);
        });
    }

    disable() {
        this._indicateMiniLed.destroy();
        this._indicateMiniLed = null;
        this._indicatePanelOd.destroy();
        this._indicatePanelOd = null;

        this._dbus_charge.stop();
        this._dbus_supported.stop();
    }
}

//@ts-ignore
function init() {
    return new Extension();
}