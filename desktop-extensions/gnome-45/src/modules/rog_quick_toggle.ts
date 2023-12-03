import { Extension, gettext as _ } from "@girs/gnome-shell/extensions/extension";
import { addQuickSettingsItems } from "./helpers";
import { quickSettings } from "@girs/gnome-shell/ui";
import { Gio } from "@girs/gio-2.0";
import { GObject } from "@girs/gobject-2.0";
import { uuid } from "../extension";
import { DbusBase } from "./dbus_proxy";

export const AsusQuickToggle = GObject.registerClass(
  class AsusQuickToggle extends quickSettings.QuickToggle {
    dbus!: DbusBase;
    prop_name: string = "";
    public toggle_callback = () => {};

    constructor(dbus: DbusBase, prop_name: string, setting: string, title: string) {
      super({
        label: title,
        icon_name: "selection-mode-symbolic",
        toggle_mode: true,
      });
      this.prop_name = prop_name;
      this.label = title;
      this.dbus = dbus;

      this.dbus?.proxy.connect("g-properties-changed", (_proxy, changed, invalidated) => {
        const properties = changed.deepUnpack();
        // .find() fails on some shit for some reason
        for (const v of Object.entries(properties)) {
          if (v[0] == this.prop_name) {
            const checked = v[1].unpack();
            if (this.checked !== checked) this.checked = checked;
            break;
          }
        }
      });

      this.connectObject("clicked", () => this._toggleMode(), this);

      this.connect("destroy", () => {
        this.destroy();
      });

      Extension.lookupByUUID(uuid)
        ?.getSettings()
        .bind(setting, this, "checked", Gio.SettingsBindFlags.DEFAULT);

      this.sync();

      addQuickSettingsItems([this]);
    }

    _toggleMode() {
      // hacky shit, index to get base object property and set it
      const checked = this.dbus.proxy[this.prop_name];
      if (this.checked !== checked) this.dbus.proxy[this.prop_name] = this.checked;
      this.toggle_callback();
    }

    sync() {
      const checked = this.dbus.proxy[this.prop_name];
      if (this.checked !== checked) this.set({ checked });
    }
  },
);
