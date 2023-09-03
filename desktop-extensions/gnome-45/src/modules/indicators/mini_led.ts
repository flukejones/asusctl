import Gio from 'gi://Gio';
import GObject from 'gi://GObject';
import * as AsusExtension from '../../extension';
import {QuickSettingsMenu, SystemIndicator} from 'resource:///org/gnome/shell/ui/quickSettings.js';

export const IndicateMiniLed = GObject.registerClass(
    class IndicateMiniLed extends SystemIndicator {
        constructor() {
            super();

            // Create the icon for the indicator
            this._indicator = this._addIndicator();
            this._indicator.icon_name = "selection-mode-symbolic";

            // Showing the indicator when the feature is enabled
            AsusExtension.extension._settings.bind("mini-led-enabled",
                this._indicator, "visible",
                Gio.SettingsBindFlags.DEFAULT);

            // Add the indicator to the panel and the toggle to the menu
            QuickSettingsMenu._indicators.add_child(this);
        }
    });