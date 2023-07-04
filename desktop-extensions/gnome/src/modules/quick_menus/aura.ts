declare const imports: any;
// REF: https://gjs.guide/extensions/development/creating.html

import { addQuickSettingsItems } from "../helpers";
import { AuraDbus } from "../dbus/aura";
import { AuraModeNum } from "../../bindings/aura";

const { GObject } = imports.gi;
const ExtensionUtils = imports.misc.extensionUtils;
// const Me = ExtensionUtils.getCurrentExtension();
// const Main = imports.ui.main;
const PopupMenu = imports.ui.popupMenu;
const QuickSettings = imports.ui.quickSettings;

export const AuraMenuToggle = GObject.registerClass(
    class AuraMenuToggle extends QuickSettings.QuickMenuToggle {
        private _dbus_aura: AuraDbus;

        constructor(dbus_aura: AuraDbus) {
            super({
                title: "Aura Modes",
                iconName: "selection-mode-symbolic",
                toggleMode: true,
            });
            this._dbus_aura = dbus_aura;

            this.connectObject(
                "destroy", () => this._settings.run_dispose(),
                this);

            this.menu.setHeader("selection-mode-symbolic", this._dbus_aura.current_aura_mode);

            this._settings = ExtensionUtils.getSettings();

            this._itemsSection = new PopupMenu.PopupMenuSection();

            this._dbus_aura.aura_modes.forEach((mode, key) => {
                this._itemsSection.addAction(key, ()=>{
                    this._dbus_aura.setLedMode(mode);
                    this.sync();
                }, "");
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
                "clicked", () => {
                    // TODO: open a configuration tool
                    this.sync();
                },
                this);

            this._dbus_aura.notifyAuraModeSubscribers.push(this);
            this.sync();

            addQuickSettingsItems([this]);
        }

        sync() {
            const checked = this._dbus_aura.current_aura_mode != AuraModeNum.Static;
            this.title = this._dbus_aura.current_aura_mode;

            if (this.checked !== checked)
                this.set({ checked });
        }
    });
