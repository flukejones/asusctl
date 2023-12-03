import { Extension, gettext as _ } from "@girs/gnome-shell/extensions/extension";
import { quickSettings, popupMenu } from "@girs/gnome-shell/ui";
import { GObject } from "@girs/gobject-2.0";

import { DbusBase } from "../../modules/dbus_proxy";

import { addQuickSettingsItems } from "../helpers";

import * as AsusExtension from "../../extension";
import * as platform from "../../bindings/platform";
import { uuid } from "../../extension";
import { AsusMenuToggle } from "../rog_menu_toggle";

export const FeatureMenuToggle = GObject.registerClass(
  class FeatureMenuToggle extends quickSettings.QuickMenuToggle {
    private dbus_platform: DbusBase;
    private dbus_anime: DbusBase;
    private last_selection = "mini-led";
    private supported_properties!: platform.Properties;
    private supported_interfaces: string[] = [];

    private miniLed?: typeof AsusMenuToggle;
    private panelOd?: typeof AsusMenuToggle;
    private animeDisplayPower?: typeof AsusMenuToggle;
    private animePowersaveAnim?: typeof AsusMenuToggle;
    _itemsSection: popupMenu.PopupMenuSection;

    constructor(dbus_platform: DbusBase, dbus_anime: DbusBase) {
      super({
        label: "Laptop",
        toggle_mode: true,
        icon_name: "selection-mode-symbolic",
      });
      this.label = "Laptop";
      this.title = "Laptop";
      this.dbus_platform = dbus_platform;
      this.dbus_anime = dbus_anime;

      this.menu.setHeader("selection-mode-symbolic", "Laptop features");

      this.last_selection = Extension.lookupByUUID(AsusExtension.uuid)
        ?.getSettings()
        .get_string("primary-quickmenu-toggle")!;

      this.supported_interfaces = this.dbus_platform?.proxy.SupportedInterfacesSync()[0];
      this.supported_properties = this.dbus_platform?.proxy.SupportedPropertiesSync()[0];

      // TODO: temporary block
      if (this.last_selection == "mini-led" && !this.supported_properties.includes("MiniLed")) {
        this.last_selection = "panel-od";
      } else if (
        this.last_selection == "panel-od" &&
        !this.supported_properties.includes("PanelOd")
      ) {
        this.last_selection = "anime-power";
      } else if (
        this.last_selection == "anime-power" &&
        !this.supported_interfaces.includes("Anime")
      ) {
        this.last_selection = "mini-led";
      } else if (this.last_selection.length == 0) {
        this.last_selection = "panel-od";
      }

      // AsusExtension.extension._settings.connect('changed::primary-quickmenu-toggle', this.sync);
      Extension.lookupByUUID(uuid)
        ?.getSettings()
        .set_string("primary-quickmenu-toggle", this.last_selection);

      this._itemsSection = new popupMenu.PopupMenuSection();
      if (this.supported_properties.includes("MiniLed")) {
        if (this.miniLed == null) {
          this.miniLed = new AsusMenuToggle(
            this.dbus_platform,
            "MiniLed",
            "mini-led-enabled",
            "Mini-LED Enabled",
          );
          this._itemsSection.addMenuItem(this.miniLed, 0);
          this.miniLed.toggle_callback = () => {
            this.last_selection = "mini-led";
          };
        }
      }

      if (this.supported_properties.includes("PanelOd")) {
        if (this.panelOd == null) {
          this.panelOd = new AsusMenuToggle(
            this.dbus_platform,
            "PanelOd",
            "panel-od-enabled",
            "Panel Overdrive Enabled",
          );
          this._itemsSection.addMenuItem(this.panelOd, 1);
          this.panelOd.toggle_callback = () => {
            this.last_selection = "panel-od";
          };
        }
      }

      if (this.supported_interfaces.includes("Anime")) {
        if (this.animeDisplayPower == null) {
          this.animeDisplayPower = new AsusMenuToggle(
            this.dbus_anime,
            "EnableDisplay",
            "anime-power",
            "AniMe Display Enabled",
          );
          this._itemsSection.addMenuItem(this.animeDisplayPower, 2);
          this.animeDisplayPower.toggle_callback = () => {
            this.last_selection = "anime-power";
          };
        }

        if (this.animePowersaveAnim == null) {
          this.animePowersaveAnim = new AsusMenuToggle(
            this.dbus_anime,
            "BuiltinsEnabled",
            "anime-builtins",
            "AniMe Built-in Animations",
          );
          this._itemsSection.addMenuItem(this.animePowersaveAnim, 3);
          this.animePowersaveAnim.toggle_callback = () => {
            this.last_selection = "anime-builtins";
          };
        }
      }

      this.connectObject(
        "clicked",
        () => {
          this._toggle();
        },
        this,
      );

      this.menu.addMenuItem(this._itemsSection, 0);

      this.dbus_platform?.proxy.connect("g-properties-changed", (_proxy, changed, invalidated) => {
        //const properties = changed.deepUnpack();
        this.sync();
      });

      this.dbus_anime?.proxy.connect("g-properties-changed", (_proxy, changed, invalidated) => {
        //const properties = changed.deepUnpack();
        this.sync();
      });

      // // Add an entry-point for more extension._settings
      // this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());
      // const settingsItem = this.menu.addAction("More Settings",
      //     () => ExtensionUtils.openPrefs());
      // // Ensure the extension._settings are unavailable when the screen is locked
      // settingsItem.visible = Main.sessionMode.allowSettings;
      // this.menu._settingsActions[Me.uuid] = settingsItem;

      this.sync();
      addQuickSettingsItems([this]);
    }

    _toggle() {
      if (this.last_selection == "mini-led" && this.miniLed != null) {
        if (this.checked !== this.dbus_platform.proxy.MiniLed)
          this.dbus_platform.proxy.MiniLed = this.checked;
      }

      if (this.last_selection == "panel-od" && this.panelOd != null) {
        if (this.checked !== this.dbus_platform.proxy.PanelOd) {
          this.dbus_platform.proxy.PanelOd = this.checked;
        }
      }

      if (this.last_selection == "anime-power" && this.animeDisplayPower != null) {
        if (this.checked !== this.dbus_anime.proxy.EnableDisplay)
          this.dbus_anime.proxy.EnableDisplay = this.checked;
      }

      if (this.last_selection == "anime-builtins" && this.animePowersaveAnim != null) {
        if (this.checked !== this.dbus_anime.proxy.BuiltinsEnabled)
          this.dbus_anime.proxy.BuiltinsEnabled = this.checked;
      }
    }

    sync() {
      let checked = false;
      if (this.last_selection == "mini-led" && this.miniLed != null) {
        this.title = this.miniLed.title;
        checked = this.dbus_platform.proxy.MiniLed;
      }

      if (this.last_selection == "panel-od" && this.panelOd != null) {
        this.title = this.panelOd.title;
        checked = this.dbus_platform.proxy.PanelOd;
      }

      if (this.last_selection == "anime-power" && this.animeDisplayPower != null) {
        this.title = this.animeDisplayPower.title;
        checked = this.dbus_anime.proxy.EnableDisplay;
      }

      if (this.last_selection == "anime-builtins" && this.animePowersaveAnim != null) {
        this.title = this.animePowersaveAnim.title;
        checked = this.dbus_anime.proxy.BuiltinsEnabled;
      }

      // if (this.animePowersaveAnim != null) {
      // }

      if (this.checked !== checked) this.set({ checked });
    }

    destroy() {
      // this.panelOd?.destroy();
    }
  },
);
