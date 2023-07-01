declare const imports: any;
// REF: https://gjs.guide/extensions/development/creating.html

const { GObject, Gio } = imports.gi;
const ExtensionUtils = imports.misc.extensionUtils;

const { SystemIndicator } = imports.ui.quickSettings;
const QuickSettingsMenu = imports.ui.main.panel.statusArea.quickSettings;

import { Platform } from '../dbus/platform';
import { QuickMiniLed } from '../quick_toggles/mini_led';

export const IndicateMiniLed = GObject.registerClass(
    class IndicateMiniLed extends SystemIndicator {
        constructor(dbus_platform: Platform) {
            super();

            // Create the icon for the indicator
            this._indicator = this._addIndicator();
            this._indicator.icon_name = 'selection-mode-symbolic';

            // Showing the indicator when the feature is enabled
            this._settings = ExtensionUtils.getSettings();
            this._settings.bind('mini-led-enabled',
                this._indicator, 'visible',
                Gio.SettingsBindFlags.DEFAULT);

            // // Create the toggle and associate it with the indicator, being sure to
            // // destroy it along with the indicator
            // this.quickSettingsItems.push(new QuickMiniLed(dbus_platform));

            // this.connect('destroy', () => {
            //     this.quickSettingsItems.forEach((item: { destroy: () => any; }) => item.destroy());
            // });

            // Add the indicator to the panel and the toggle to the menu
            QuickSettingsMenu._indicators.add_child(this);
        }
    });