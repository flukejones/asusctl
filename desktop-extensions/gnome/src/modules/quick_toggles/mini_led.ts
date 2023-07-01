declare const imports: any;

import { Platform } from "../dbus/platform";

const { GObject, Gio } = imports.gi;
const ExtensionUtils = imports.misc.extensionUtils;

const { QuickToggle } = imports.ui.quickSettings;

export const QuickMiniLed = GObject.registerClass(
    class QuickMiniLed extends QuickToggle {
        private _dbus_platform: Platform;

        constructor(dbus_platform: Platform) {
            super({
                title: 'MiniLED',
                iconName: 'selection-mode-symbolic',
                toggleMode: true,
            });
            this._dbus_platform = dbus_platform;
            this.label = 'MiniLED';
            this._settings = ExtensionUtils.getSettings();

            this.connectObject(
                'destroy', () => this._settings.run_dispose(),
                'clicked', () => this._toggleMode(),
                this);

            this.connect('destroy', () => {
                this.destroy();
            });

            this._settings.bind('mini-led-enabled',
                this, 'checked',
                Gio.SettingsBindFlags.DEFAULT);

            this._sync();
        }

        _toggleMode() {
            this._dbus_platform.setMiniLedMode(this.checked);
            this._sync();
        }

        _sync() {
            const checked = this._dbus_platform.getMiniLedMode();
            if (this.checked !== checked)
                this.set({ checked });
        }
    });