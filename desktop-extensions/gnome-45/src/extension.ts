import { Extension, gettext as _ } from "@girs/gnome-shell/extensions/extension";
import * as platform from "./bindings/platform";
import { AsusQuickToggle } from "./modules/rog_quick_toggle";
import { AsusMenuToggle } from "./modules/rog_menu_toggle";
import { AsusIndicator } from "./modules/rog_indicator";
import { AsusSlider } from "./modules/rog_slider_100pc";
import { FeatureMenuToggle } from "./modules/quick_menus/laptop_features";
import { DbusBase } from "./modules/dbus_proxy";
import { main } from "@girs/gnome-shell/ui";

export const uuid = "asusctl-gnome@asus-linux.org";
export default class AsusExtension extends Extension {
  // public dbus_aura: AuraDbus = new AuraDbus;
  // public dbus_anime: AnimeDbus = new AnimeDbus;
  public dbus_platform: DbusBase | undefined;
  public dbus_anime: DbusBase | undefined;

  private individual = false;
  public supported_properties!: platform.Properties;
  public supported_interfaces: string[] = [];
  private feature_menu = null;
  private panel_od = null;
  private mini_led = null;
  private anime_display = null;
  private anime_builtins = null;
  private charge_thres = null;
  // private _feature: typeof FeatureMenuToggle;

  async enable() {
    log(this.path);

    if (this.dbus_platform == undefined) {
      this.dbus_platform = new DbusBase("org-asuslinux-platform-4.xml", "/org/asuslinux/Platform");
      await this.dbus_platform.start();
    }

    if (this.dbus_anime == undefined) {
      this.dbus_anime = new DbusBase("org-asuslinux-anime-4.xml", "/org/asuslinux/Anime");
      await this.dbus_anime.start();
    }

    this.supported_interfaces = this.dbus_platform?.proxy.SupportedInterfacesSync()[0];
    this.supported_properties = this.dbus_platform?.proxy.SupportedPropertiesSync()[0];
    log(this.supported_interfaces);
    log(this.supported_properties);

    // new AsusIndicator("selection-mode-symbolic", "mini-led-enabled");
    // new AsusIndicator("selection-mode-symbolic", "panel-od-enabled");

    if (!this.individual) {
      if (this.feature_menu == null)
        this.feature_menu = new FeatureMenuToggle(this.dbus_platform, this.dbus_anime);
    } else {
      if (this.supported_properties.includes("PanelOd") && this.dbus_platform.proxy.PanelOd != null)
        if (this.panel_od == null) {
          this.panel_od = new AsusQuickToggle(
            this.dbus_platform,
            "PanelOd",
            "panel-od-enabled",
            "Panel Overdrive",
          );
        }

      if (this.supported_properties.includes("MiniLed") && this.dbus_platform.proxy.MiniLed != null)
        if (this.mini_led == null) {
          this.mini_led = new AsusQuickToggle(
            this.dbus_platform,
            "MiniLed",
            "mini-led-enabled",
            "Mini-LED",
          );
        }

      if (
        this.supported_interfaces.includes("Anime") &&
        this.dbus_anime.proxy.EnableDisplay != null
      )
        if (this.anime_display == null) {
          this.anime_display = new AsusQuickToggle(
            this.dbus_anime,
            "EnableDisplay",
            "anime-power",
            "AniMe Display",
          );
        }

      if (
        this.supported_interfaces.includes("Anime") &&
        this.dbus_anime.proxy.BuiltinsEnabled != null
      )
        if (this.anime_builtins == null) {
          this.anime_builtins = new AsusQuickToggle(
            this.dbus_anime,
            "BuiltinsEnabled",
            "anime-builtins",
            "Use builtins",
          );
        }
    }

    if (
      this.supported_properties.includes("ChargeControlEndThreshold") &&
      this.dbus_platform.proxy.ChargeControlEndThreshold != null
    )
      if (this.charge_thres == null) {
        this.charge_thres = new AsusSlider(
          this.dbus_platform,
          "ChargeControlEndThreshold",
          "charge-level",
          "Charge Level",
        );
      }
  }

  disable() {
    this.dbus_platform?.stop();
    this.dbus_anime?.stop();

    this.feature_menu?.destroy();
    feature_menu?.destroy();
    panel_od?.destroy();
    mini_led?.destroy();
    anime_display?.destroy();
    anime_builtins?.destroy();
    charge_thres?.destroy();
  }
}
