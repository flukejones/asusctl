import { popupMenu } from "@girs/gnome-shell/ui";
import { GObject } from "@girs/gobject-2.0";
import { DbusBase } from "./dbus_proxy";

export const AsusMenuToggle = GObject.registerClass(
  class AsusMenuToggle extends popupMenu.PopupSwitchMenuItem {
    public title: string = "";
    dbus!: DbusBase;
    prop_name: string = "";
    public toggle_callback = () => {};

    constructor(dbus: DbusBase, prop_name: string, setting: string, title: string) {
      super(title, true);
      this.prop_name = prop_name;
      this.dbus = dbus;
      this.title = title;

      this.dbus?.proxy.connect("g-properties-changed", (_proxy, changed, invalidated) => {
        const properties = changed.deepUnpack();
        // .find() fails on some shit for some reason
        for (const v of Object.entries(properties)) {
          if (v[0] == this.prop_name) {
            this.sync();
            break;
          }
        }
      });

      this.connectObject("toggled", () => this._toggleMode(), this);

      this.connect("destroy", () => {
        this.destroy();
      });

      this.sync();
    }

    _toggleMode() {
      // hacky shit, index to get base object property and set it
      const state = this.dbus.proxy[this.prop_name];
      if (this.state !== state) this.dbus.proxy[this.prop_name] = this.state;
      this.toggle_callback();
    }

    sync() {
      const state = this.dbus.proxy[this.prop_name];
      if (this.state !== state) this.setToggleState(state);
    }
  },
);
