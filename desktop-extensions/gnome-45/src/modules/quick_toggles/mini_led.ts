import { Platform } from "../dbus/platform";
import { addQuickSettingsItems } from "../helpers";
import Gio from 'gi://Gio';
import GObject from 'gi://GObject';
import * as QuickToggle from 'resource:///org/gnome/shell/ui/quickSettings.js';
import * as AsusExtension from "../../extension.js";

export const QuickMiniLed = GObject.registerClass(
    class QuickMiniLed extends QuickToggle {
        private _dbus_platform: Platform;

        constructor(dbus_platform: Platform) {
            super({
                title: "MiniLED",
                iconName: "selection-mode-symbolic",
                toggleMode: true,
            });
            this._dbus_platform = dbus_platform;
            this.label = "MiniLED";

            this.connectObject(
                "destroy", () =>  AsusExtension.extension._settings.run_dispose(),
                "clicked", () => this._toggleMode(),
                this);

            this.connect("destroy", () => {
                this.destroy();
            });

            AsusExtension.extension._settings.bind("mini-led-enabled",
                this, "checked",
                Gio.SettingsBindFlags.DEFAULT);

            this.sync();

            addQuickSettingsItems([this]);
        }

        _toggleMode() {
            const checked = this._dbus_platform.getMiniLedMode();
            if (this.checked !== checked)
                this._dbus_platform.setMiniLedMode(this.checked);
        }

        sync() {
            const checked = this._dbus_platform.getMiniLedMode();
            if (this.checked !== checked)
                this.set({ checked });
        }
    });