declare const imports: any;
var extensionInstance: any;
//@ts-ignore
// const Me = imports.misc.extensionUtils.getCurrentExtension();

// REF: https://gjs.guide/extensions/development/creating.html

const { QuickToggle } = imports.ui.quickSettings;
const QuickSettingsMenu = imports.ui.main.panel.statusArea.quickSettings;

import { AnimeDbus } from './modules/dbus/animatrix';
import { Power } from './modules/dbus/power';
import { Supported } from './modules/dbus/supported';
import { Platform } from './modules/dbus/platform';

import { QuickPanelOd } from './modules/quick_toggles/panel_od';
import { IndicateMiniLed } from './modules/indicators/mini_led';
import { QuickMiniLed } from './modules/quick_toggles/mini_led';


function addQuickSettingsItems(items: [typeof QuickToggle]) {
    // Add the items with the built-in function
    QuickSettingsMenu._addItems(items);

    // Ensure the tile(s) are above the background apps menu
    for (const item of items) {
        QuickSettingsMenu.menu._grid.set_child_below_sibling(item,
            QuickSettingsMenu._backgroundApps.quickSettingsItems[0]);
    }
}

class Extension {
    private _indicateMiniLed: typeof IndicateMiniLed;
    private _quickMiniLed: typeof QuickMiniLed;
    private _quickPanelOd: typeof QuickPanelOd;
    private _dbus_power!: Power;
    private _dbus_anime!: AnimeDbus;

    public dbus_platform!: Platform;
    public dbus_supported!: Supported;

    constructor() {
        this._indicateMiniLed = null;
        this._quickMiniLed = null;
        this._quickPanelOd = null;

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
                addQuickSettingsItems([this._quickMiniLed]);
            }
            if (this._indicateMiniLed == null) {
                this._indicateMiniLed = new IndicateMiniLed(this.dbus_platform);
            }
        }
        if (this.dbus_supported.supported.rog_bios_ctrl.panel_overdrive) {
            if (this._quickPanelOd == null) {
                this._quickPanelOd = new QuickPanelOd(this.dbus_platform);
                addQuickSettingsItems([this._quickPanelOd]);
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

        this._dbus_power.stop();
        this.dbus_platform.stop();
        this._dbus_anime.stop();
        this.dbus_supported.stop();
    }
}

//@ts-ignore
function init() {
    extensionInstance = new Extension();
    return extensionInstance;
}