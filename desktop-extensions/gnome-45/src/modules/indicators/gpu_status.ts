import GObject from 'gi://GObject';
import {QuickSettingsMenu, SystemIndicator} from 'resource:///org/gnome/shell/ui/quickSettings.js';

export const IndicateGpuStatus = GObject.registerClass(
    class IndicateGpuStatus extends SystemIndicator {
        constructor() {
            super();

            // Create the icon for the indicator
            this._indicator = this._addIndicator();
            this._indicator.icon_name = "selection-mode-symbolic";
            this._indicator.visible = true;

            this.sync();
            // Add the indicator to the panel and the toggle to the menu
            QuickSettingsMenu._indicators.add_child(this);
        }

        sync() {
            // TODO:
        }
    });