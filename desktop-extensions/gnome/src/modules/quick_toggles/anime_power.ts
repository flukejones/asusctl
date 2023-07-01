declare const imports: any;

import { AnimeDbus } from "../dbus/animatrix";
import { addQuickSettingsItems } from "../helpers";

const { GObject, Gio } = imports.gi;
const ExtensionUtils = imports.misc.extensionUtils;

const { QuickToggle } = imports.ui.quickSettings;

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
            this._settings = ExtensionUtils.getSettings();

            this.connectObject(
                "destroy", () => this._settings.run_dispose(),
                "clicked", () => this._toggleMode(),
                this);

            this.connect("destroy", () => {
                this.destroy();
            });

            this._settings.bind("anime-power",
                this, "checked",
                Gio.SettingsBindFlags.DEFAULT);

            this._sync();

            addQuickSettingsItems([this]);
        }

        _toggleMode() {
            this._dbus_anime.setEnableDisplay(this.checked);
            this._sync();
        }

        _sync() {
            this._dbus_anime.getDeviceState();
            const checked = this._dbus_anime.deviceState.display_enabled;
            if (this.checked !== checked)
                this.set({ checked });
        }
    });