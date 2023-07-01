declare const imports: any;

const { QuickToggle } = imports.ui.quickSettings;
const QuickSettingsMenu = imports.ui.main.panel.statusArea.quickSettings;

export function addQuickSettingsItems(items: [typeof QuickToggle], width = 1) {
    // Add the items with the built-in function
    QuickSettingsMenu._addItems(items, width);

    // Ensure the tile(s) are above the background apps menu
    for (const item of items) {
        QuickSettingsMenu.menu._grid.set_child_below_sibling(item,
            QuickSettingsMenu._backgroundApps.quickSettingsItems[0]);
    }
}