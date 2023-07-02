declare const imports: any;

import { AnimeDbus } from "../dbus/animatrix";

const { GObject, Gio } = imports.gi;
const ExtensionUtils = imports.misc.extensionUtils;
const PopupMenu = imports.ui.popupMenu;

export const MenuToggleAnimePower = GObject.registerClass(
    class MenuToggleAnimePower extends PopupMenu.PopupSwitchMenuItem {
        private _dbus_anime: AnimeDbus;

        constructor(dbus_anime: AnimeDbus) {
            super(
                "AniMatrix Power", dbus_anime.deviceState.display_enabled
            );
            this._dbus_anime = dbus_anime;
            this.label = "AniMatrix Power";
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
            if (this.state !== this._dbus_anime.getDeviceState())
                this._dbus_anime.setEnableDisplay(this.state);
        }

        sync() {
            this._dbus_anime.getDeviceState();
            const checked = this._dbus_anime.deviceState.display_enabled;
            this.setToggleState(checked);
        }
    });