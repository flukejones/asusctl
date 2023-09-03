declare const imports: any;

import { Platform } from "../dbus/platform";

const { GObject, Gio } = imports.gi;
const ExtensionUtils = imports.misc.extensionUtils;
const PopupMenu = imports.ui.popupMenu;

export const MenuToggleMiniLed = GObject.registerClass(
    class MenuToggleMiniLed extends PopupMenu.PopupSwitchMenuItem {
        private _dbus_platform: Platform;
        public toggle_callback = () => {};

        constructor(dbus_platform: Platform) {
            super("MiniLED", dbus_platform.bios.mini_led_mode);

            this._dbus_platform = dbus_platform;
            this.label = "MiniLED";
            this._settings = ExtensionUtils.getSettings();

            this.connectObject(
                "destroy", () => this._settings.run_dispose(),
                "toggled", () => this._toggleMode(),
                this);

            this.connect("destroy", () => {
                this.destroy();
            });

            this.sync();
        }

        _toggleMode() {
            this._dbus_platform.getMiniLedMode();
            const state = this._dbus_platform.bios.mini_led_mode;
            if (this.state !== state)
                this._dbus_platform.setMiniLedMode(this.state);
            this.toggle_callback();
        }

        sync() {
            this._dbus_platform.getMiniLedMode();
            const toggled = this._dbus_platform.bios.mini_led_mode;
            this.setToggleState(toggled);
        }
    });