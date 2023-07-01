declare const imports: any;
// REF: https://gjs.guide/extensions/development/creating.html

const { GObject, Gio } = imports.gi;
const ExtensionUtils = imports.misc.extensionUtils;

const { SystemIndicator } = imports.ui.quickSettings;
const QuickSettingsMenu = imports.ui.main.panel.statusArea.quickSettings;

export const IndicateMiniLed = GObject.registerClass(
    class IndicateMiniLed extends SystemIndicator {
        constructor() {
            super();

            // Create the icon for the indicator
            this._indicator = this._addIndicator();
            this._indicator.icon_name = 'selection-mode-symbolic';

            // Showing the indicator when the feature is enabled
            this._settings = ExtensionUtils.getSettings();
            this._settings.bind('mini-led-enabled',
                this._indicator, 'visible',
                Gio.SettingsBindFlags.DEFAULT);

            // Add the indicator to the panel and the toggle to the menu
            QuickSettingsMenu._indicators.add_child(this);
        }
    });