import { Extension, gettext as _ } from "@girs/gnome-shell/extensions/extension";
import { addQuickSettingsItems } from "./helpers";
import { quickSettings } from "@girs/gnome-shell/ui";
import { Gio } from "@girs/gio-2.0";
import { GObject } from "@girs/gobject-2.0";
import { uuid } from "../extension";
import { DbusBase } from "./dbus_proxy";

export const AsusSlider = GObject.registerClass(
  class AsusSlider extends quickSettings.QuickSlider {
    private dbus: DbusBase;
    private settings: any = undefined;
    private setting = "";
    private prop_name = "";

    constructor(dbus: DbusBase, prop_name: string, setting: string, title: string) {
      super({
        label: title,
        icon_name: "selection-mode-symbolic",
      });
      this.label = title;
      this.dbus = dbus;
      this.setting = setting;
      this.prop_name = prop_name;
      this.settings = Extension.lookupByUUID(uuid)?.getSettings();

      this._sliderChangedId = this.slider.connect("drag-end", this._onSliderChanged.bind(this));

      // Binding the slider to a GSettings key

      this.settings.connect(`changed::${this.setting}`, this._onSettingsChanged.bind(this));

      // Set an accessible name for the slider
      this.slider.accessible_name = title;

      this.dbus?.proxy.connect("g-properties-changed", (_proxy, changed, invalidated) => {
        const properties = changed.deepUnpack();
        // .find() fails on some shit for some reason
        for (const v of Object.entries(properties)) {
          if (v[0] == this.prop_name) {
            const checked = v[1].unpack();
            this._sync();
            break;
          }
        }
      });

      this._sync();
      this._onSettingsChanged();

      addQuickSettingsItems([this], 2);
    }

    _onSettingsChanged() {
      // Prevent the slider from emitting a change signal while being updated
      this.slider.block_signal_handler(this._sliderChangedId);
      this.slider.value = this.settings.get_uint(this.setting) / 100.0;
      this.slider.unblock_signal_handler(this._sliderChangedId);
    }

    _onSliderChanged() {
      // Assuming our GSettings holds values between 0..100, adjust for the
      // slider taking values between 0..1
      const percent = Math.floor(this.slider.value * 100);
      const stored = Math.floor(this.settings.get_uint(this.setting) / 100.0);
      if (this.slider.value !== stored) this.dbus.proxy[this.prop_name] = percent;
      this.settings.set_uint(this.setting, percent);
    }

    _sync() {
      const value = this.dbus.proxy[this.prop_name];
      if (this.slider.value !== value / 100) this.settings.set_uint(this.setting, value);
    }
  },
);
