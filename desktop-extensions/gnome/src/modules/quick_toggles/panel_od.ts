declare const imports: any;

import { Platform } from "../dbus/platform";
import { addQuickSettingsItems } from "../helpers";

const { GObject, Gio } = imports.gi;
const ExtensionUtils = imports.misc.extensionUtils;

const { QuickToggle } = imports.ui.quickSettings;

export const QuickPanelOd = GObject.registerClass(
    class QuickPanelOd extends QuickToggle {
        private _dbus_platform: Platform;

        constructor(dbus_platform: Platform) {
            super({
                title: "Panel Overdrive",
                iconName: "selection-mode-symbolic",
                toggleMode: true,
            });
            this._dbus_platform = dbus_platform;
            this.label = "Panel Overdrive";
            this._settings = ExtensionUtils.getSettings();

            this.connectObject(
                "destroy", () => this._settings.run_dispose(),
                "clicked", () => this._toggleMode(),
                this);

            this.connect("destroy", () => {
                this.destroy();
            });

            this._settings.bind("panel-od-enabled",
                this, "checked",
                Gio.SettingsBindFlags.DEFAULT);

            this._sync();

            addQuickSettingsItems([this]);
        }

        _toggleMode() {
            this._dbus_platform.setPanelOd(this.checked);
            this._sync();
        }

        _sync() {
            const checked = this._dbus_platform.getPanelOd();
            if (this.checked !== checked)
                this.set({ checked });
        }
    });