import { Extension, gettext as _ } from "@girs/gnome-shell/extensions/extension";
import { quickSettings, main } from "@girs/gnome-shell/ui";
import { Gio } from "@girs/gio-2.0";
import { GObject } from "@girs/gobject-2.0";
import { uuid } from "../extension";
//import { DbusBase } from '../dbus_proxy';

export const AsusIndicator = GObject.registerClass(
  class AsusIndicator extends quickSettings.SystemIndicator {
    private _indicator: any;
    private _settings: Gio.Settings | undefined;

    constructor(icon_name: string, setting_name: string) {
      super();
      // Create an icon for the indicator
      this._indicator = this._addIndicator();
      this._indicator.icon_name = icon_name;

      // Showing an indicator when the feature is enabled
      this._settings = Extension.lookupByUUID(uuid)?.getSettings();
      this._settings?.bind(setting_name, this._indicator, "visible", Gio.SettingsBindFlags.DEFAULT);

      main.panel.statusArea.quickSettings.addExternalIndicator(this);
    }
  },
);
