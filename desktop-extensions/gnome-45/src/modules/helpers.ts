import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import {QuickSettingsMenu, QuickToggle } from 'resource:///org/gnome/shell/ui/quickSettings.js';

export function addQuickSettingsItems(items: [typeof QuickToggle], width = 1) {
    const QuickSettingsMenu = Main.panel.statusArea.quickSettings;
    // Add the items with the built-in function
    QuickSettingsMenu._addItems(items, width);

    // Ensure the tile(s) are above the background apps menu
    for (const item of items) {
        QuickSettingsMenu.menu._grid.set_child_below_sibling(item,
            QuickSettingsMenu._backgroundApps.quickSettingsItems[0]);
    }
}