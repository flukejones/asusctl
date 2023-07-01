declare const imports: any;

import { Platform } from "../dbus/platform";

const { GObject, Gio } = imports.gi;
const ExtensionUtils = imports.misc.extensionUtils;
const PopupMenu = imports.ui.popupMenu;

export const MenuTogglePanelOd = GObject.registerClass(
    class MenuTogglePanelOd extends PopupMenu.PopupSwitchMenuItem {
        private _dbus_platform: Platform;

        constructor(dbus_platform: Platform) {
            super("Panel Overdrive", dbus_platform.bios.panel_overdrive);

            this._dbus_platform = dbus_platform;
            this.label = "Panel Overdrive";
            this._settings = ExtensionUtils.getSettings();

            this.connectObject(
                "destroy", () => this._settings.run_dispose(),
                "toggled", () => this._toggleMode(),
                this);

            this.connect("destroy", () => {
                this.destroy();
            });

            this._settings.bind("panel-od-enabled",
                this, "toggled",
                Gio.SettingsBindFlags.DEFAULT);

            this.sync();
        }

        _toggleMode() {
            this._dbus_platform.getPanelOd();
            const state = this._dbus_platform.bios.panel_overdrive;
            if (this.state !== state)
                this._dbus_platform.setPanelOd(this.state);
        }

        sync() {
            this._dbus_platform.getPanelOd();
            const toggled = this._dbus_platform.bios.panel_overdrive;
            this.setToggleState(toggled);
        }
    });