import { addQuickSettingsItems } from "../helpers";
import { AuraDbus } from "../dbus/aura";
import { AuraEffect, AuraModeNum } from "../../bindings/aura";
import GObject from "gi://GObject";

import * as PopupMenu from "resource:///org/gnome/shell/ui/popupMenu.js";
import * as QuickSettings from "resource:///org/gnome/shell/ui/quickSettings.js";

export const AuraMenuToggle = GObject.registerClass(
  class AuraMenuToggle extends QuickSettings.QuickMenuToggle {
    private _dbus_aura: AuraDbus;
    private _last_mode: AuraModeNum = AuraModeNum.Static;

    constructor(dbus_aura: AuraDbus) {
      super({
        title: "Aura Modes",
        iconName: "selection-mode-symbolic",
        toggleMode: true,
      });
      this._dbus_aura = dbus_aura;

      this.connectObject(this);

      this.menu.setHeader("selection-mode-symbolic", this._dbus_aura.current_aura_mode);

      this._itemsSection = new PopupMenu.PopupMenuSection();

      this._dbus_aura.aura_modes.forEach((mode, key) => {
        this._itemsSection.addAction(
          key,
          () => {
            this._dbus_aura.setLedMode(mode);
            this.sync();
          },
          "",
        );
      });

      this.menu.addMenuItem(this._itemsSection);

      // Add an entry-point for more settings
      // this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());
      // const settingsItem = this.menu.addAction("More Settings",
      //     () => ExtensionUtils.openPrefs());
      // // Ensure the settings are unavailable when the screen is locked
      // settingsItem.visible = Main.sessionMode.allowSettings;
      // this.menu._settingsActions[Me.uuid] = settingsItem;

      this.connectObject(
        "clicked",
        () => {
          let mode: AuraEffect | undefined;
          if (this._dbus_aura.current_aura_mode == AuraModeNum.Static) {
            mode = this._dbus_aura.aura_modes.get(this._last_mode);
          } else {
            mode = this._dbus_aura.aura_modes.get(AuraModeNum.Static);
          }
          if (mode != undefined) {
            this._dbus_aura.setLedMode(mode);
            this.sync();
          }
        },
        this,
      );

      this._dbus_aura.notifyAuraModeSubscribers.push(this);
      this.sync();

      addQuickSettingsItems([this]);
    }

    sync() {
      const checked = this._dbus_aura.current_aura_mode != AuraModeNum.Static;
      this.title = this._dbus_aura.current_aura_mode;
      if (
        this._last_mode != this._dbus_aura.current_aura_mode &&
        this._dbus_aura.current_aura_mode != AuraModeNum.Static
      ) {
        this._last_mode = this._dbus_aura.current_aura_mode;
      }

      if (this.checked !== checked) this.set({ checked });
    }
  },
);
