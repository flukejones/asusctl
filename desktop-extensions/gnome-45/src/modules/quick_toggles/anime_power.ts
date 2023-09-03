import { AnimeDbus } from "../dbus/animatrix";
import { addQuickSettingsItems } from "../helpers";

import Gio from 'gi://Gio';
import GObject from 'gi://GObject';
import * as QuickToggle from 'resource:///org/gnome/shell/ui/quickSettings.js';

import * as AsusExtension from "../../extension.js";

export const QuickAnimePower = GObject.registerClass(
    class QuickAnimePower extends QuickToggle {
        private _dbus_anime: AnimeDbus;

        constructor(dbus_anime: AnimeDbus) {
            super({
                title: "AniMatrix Power",
                iconName: "selection-mode-symbolic",
                toggleMode: true,
            });
            this._dbus_anime = dbus_anime;
            this.label = "AniMatrix Power";

            this.connectObject(
                "clicked", () => this._toggleMode(),
                this);

            this.connect("destroy", () => {
                this.destroy();
            });

            AsusExtension.extension._settings.bind("anime-power",
                this, "checked",
                Gio.SettingsBindFlags.DEFAULT);

            this.sync();

            addQuickSettingsItems([this]);
        }

        _toggleMode() {
            this._dbus_anime.getDeviceState();
            const checked = this._dbus_anime.deviceState.display_enabled;
            if (this.checked !== checked)
                this._dbus_anime.setEnableDisplay(this.checked);
        }

        sync() {
            this._dbus_anime.getDeviceState();
            const checked = this._dbus_anime.deviceState.display_enabled;
            if (this.checked !== checked)
                this.set({ checked });
        }
    });