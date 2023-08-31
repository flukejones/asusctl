import { Platform } from "../dbus/platform";
import { addQuickSettingsItems } from "../helpers";
import Gio from 'gi://Gio';
import GObject from 'gi://GObject';
import * as QuickToggle from 'resource:///org/gnome/shell/ui/quickSettings.js';
import * as AsusExtension from "../../extension.js";

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

            this.connectObject(
                "clicked", () => this._toggleMode(),
                this);

            this.connect("destroy", () => {
                this.destroy();
            });

            AsusExtension.extension._settings.bind("panel-od-enabled",
                this, "checked",
                Gio.SettingsBindFlags.DEFAULT);

            this.sync();

            addQuickSettingsItems([this]);
        }

        _toggleMode() {
            const checked = this._dbus_platform.getPanelOd();
            if (this.checked !== checked)
                this._dbus_platform.setPanelOd(this.checked);
        }

        sync() {
            const checked = this._dbus_platform.getPanelOd();
            if (this.checked !== checked)
                this.set({ checked });
        }
    });