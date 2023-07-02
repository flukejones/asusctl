declare const imports: any;
// REF: https://gjs.guide/extensions/development/creating.html

import { AnimeDbus } from "./dbus/animatrix";
import { Supported } from "./dbus/supported";
import { Platform } from "./dbus/platform";

import { addQuickSettingsItems } from "./helpers";
import { MenuToggleAnimeBuiltins, MenuToggleAnimePower } from "./menu_toggles/anime";
import { MenuTogglePanelOd } from "./menu_toggles/panel_od";
import { MenuToggleMiniLed } from "./menu_toggles/mini_led";


const { GObject } = imports.gi;

const ExtensionUtils = imports.misc.extensionUtils;
const Me = ExtensionUtils.getCurrentExtension();

const Main = imports.ui.main;
const PopupMenu = imports.ui.popupMenu;
const QuickSettings = imports.ui.quickSettings;

export const FeatureMenuToggle = GObject.registerClass(
    class FeatureMenuToggle extends QuickSettings.QuickMenuToggle {
        private _dbus_supported: Supported;
        private _dbus_platform: Platform;
        private _dbus_anime: AnimeDbus;

        public miniLed: typeof MenuToggleMiniLed;
        public panelOd: typeof MenuTogglePanelOd;
        public animeDisplayPower: typeof MenuToggleAnimePower;
        public animePowersaveAnim: typeof MenuToggleAnimeBuiltins;
        private primary = "mini-led";

        constructor(dbus_supported: Supported, dbus_platform: Platform, dbus_anime: AnimeDbus) {
            super({
                title: "Laptop",
                iconName: "selection-mode-symbolic",
                toggleMode: true,
            });
            this._dbus_supported = dbus_supported;
            this._dbus_platform = dbus_platform;
            this._dbus_anime = dbus_anime;

            this.connectObject(
                "destroy", () => this._settings.run_dispose(),
                this);

            this.menu.setHeader("selection-mode-symbolic", "Laptop features");

            this._settings = ExtensionUtils.getSettings();
            this.primary = this._settings.get_string("primary-quickmenu-toggle");

            // TODO: temporary block
            if (this.primary == "mini-led" && !this._dbus_supported.supported.rog_bios_ctrl.mini_led_mode) {
                this.primary = "panel-od";
            } else if (this.primary == "panel-od" && !this._dbus_supported.supported.rog_bios_ctrl.panel_overdrive) {
                this.primary = "anime-power";
            } else if (this.primary == "anime-power" && !this._dbus_supported.supported.anime_ctrl) {
                this.primary = "mini-led";
            } else if (this.primary.length == 0) {
                this.primary = "panel-od";
            }
            this._settings.set_string("primary-quickmenu-toggle", this.primary);

            this._itemsSection = new PopupMenu.PopupMenuSection();

            if (this._dbus_supported.supported.rog_bios_ctrl.mini_led_mode) {
                if (this.miniLed == null) {
                    this.miniLed = new MenuToggleMiniLed(this._dbus_platform);
                    this._dbus_platform.notifyMiniLedSubscribers.push(this.miniLed);
                    this._itemsSection.addMenuItem(this.miniLed);

                    if (this.primary == "mini-led") {
                        // Set the togglemenu title and action
                        this.title = this.miniLed.label;

                        this.connectObject(
                            "clicked", () => {
                                const checked = this._dbus_platform.getMiniLedMode();
                                if (this.checked !== checked)
                                    this._dbus_platform.setMiniLedMode(this.checked);
                            },
                            this);
                        this.sync();
                        this._dbus_platform.notifyMiniLedSubscribers.push(this);
                    }
                }
            }

            if (this._dbus_supported.supported.rog_bios_ctrl.panel_overdrive) {
                if (this.panelOd == null) {
                    this.panelOd = new MenuTogglePanelOd(this._dbus_platform);
                    this._dbus_platform.notifyPanelOdSubscribers.push(this.panelOd);
                    this._itemsSection.addMenuItem(this.panelOd);

                    if (this.primary == "panel-od") {
                        // Set the togglemenu title and action
                        this.title = this.panelOd.label;

                        this.connectObject(
                            "clicked", () => {
                                const checked = this._dbus_platform.getPanelOd();
                                if (this.checked !== checked)
                                    this._dbus_platform.setPanelOd(this.checked);
                            },
                            this);
                        this.sync();
                        this._dbus_platform.notifyPanelOdSubscribers.push(this);
                    }
                }
            }

            if (this._dbus_supported.supported.anime_ctrl) {
                if (this.animeDisplayPower == null) {
                    this.animeDisplayPower = new MenuToggleAnimePower(this._dbus_anime);
                    this._dbus_anime.notifyAnimeStateSubscribers.push(this.animeDisplayPower);
                    this._itemsSection.addMenuItem(this.animeDisplayPower);

                    if (this.primary == "anime-power") {
                        // Set the togglemenu title and action
                        this.title = this.animeDisplayPower.label;

                        this.connectObject(
                            "clicked", () => {
                                this._dbus_anime.getDeviceState();
                                const checked = this._dbus_anime.deviceState.display_enabled;
                                if (this.checked !== checked)
                                    this._dbus_anime.setEnableDisplay(this.checked);
                            },
                            this);
                        this.sync();
                        this._dbus_anime.notifyAnimeStateSubscribers.push(this);
                    }
                }

                if (this.animePowersaveAnim == null) {
                    this.animePowersaveAnim = new MenuToggleAnimeBuiltins(this._dbus_anime);
                    this._dbus_anime.notifyAnimeStateSubscribers.push(this.animePowersaveAnim);
                    this._itemsSection.addMenuItem(this.animePowersaveAnim);
                }
            }

            this.menu.addMenuItem(this._itemsSection);

            // Add an entry-point for more settings
            this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());
            const settingsItem = this.menu.addAction("More Settings",
                () => ExtensionUtils.openPrefs());

            // Ensure the settings are unavailable when the screen is locked
            settingsItem.visible = Main.sessionMode.allowSettings;
            this.menu._settingsActions[Me.uuid] = settingsItem;

            addQuickSettingsItems([this]);
        }

        sync() {
            let checked = false;
            switch (this.primary) {
            case "mini-led":
                checked = this._dbus_platform.getMiniLedMode();
                break;
            case "panel-od":
                checked = this._dbus_platform.getPanelOd();
                break;
            case "anime-power":
                this._dbus_anime.getDeviceState();
                checked = this._dbus_anime.deviceState.display_enabled;
                break;
            default:
                return;
            }

            if (this.checked !== checked)
                this.set({ checked });
        }
    });
