declare const imports: any;

import { AnimeDbus } from "../dbus/animatrix";

const { GObject } = imports.gi;
const ExtensionUtils = imports.misc.extensionUtils;
const PopupMenu = imports.ui.popupMenu;

export const MenuToggleAnimePower = GObject.registerClass(
    class MenuToggleAnimePower extends PopupMenu.PopupSwitchMenuItem {
        private _dbus_anime: AnimeDbus;
        public toggle_callback = () => {};

        constructor(dbus_anime: AnimeDbus) {
            super(
                "AniMatrix Display Power", dbus_anime.deviceState.display_enabled
            );
            this._dbus_anime = dbus_anime;
            this.label = "AniMatrix Display Power";
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
            this._dbus_anime.getDeviceState();
            if (this.state !== this._dbus_anime.deviceState.display_enabled)
                this._dbus_anime.setEnableDisplay(this.state);
            this.toggle_callback();
        }

        sync() {
            this._dbus_anime.getDeviceState();
            const checked = this._dbus_anime.deviceState.display_enabled;
            this.setToggleState(checked);
        }
    });


export const MenuToggleAnimeBuiltins = GObject.registerClass(
    class MenuToggleAnimeBuiltins extends PopupMenu.PopupSwitchMenuItem {
        private _dbus_anime: AnimeDbus;
        public toggle_callback = () => {};

        constructor(dbus_anime: AnimeDbus) {
            super(
                "AniMatrix Powersave Animation", dbus_anime.deviceState.builtin_anims_enabled
            );
            this._dbus_anime = dbus_anime;
            this.label = "AniMatrix Powersave Animation";
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
            this._dbus_anime.getDeviceState();
            if (this.state !== this._dbus_anime.deviceState.builtin_anims_enabled)
                this._dbus_anime.setPowersaveAnim(this.state);
            this.toggle_callback();
        }

        sync() {
            this._dbus_anime.getDeviceState();
            const checked = this._dbus_anime.deviceState.display_enabled;
            this.setToggleState(checked);
        }
    });