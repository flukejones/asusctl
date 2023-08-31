# asusctl

Requires `asusd` to be installed and running.

## build and install

```
npm install
npm run build && gnome-extensions install asusctl-gnome@asus-linux.org.zip --force
npm run build && gnome-extensions enable asusctl-gnome@asus-linux.org.zip
```

You will need to restart Gnome after installing or updating

## development

```
npm run build
gnome-extensions install asusctl-gnome@asus-linux.org.zip --force
MUTTER_DEBUG_DUMMY_MODE_SPECS=1366x768 dbus-run-session -- gnome-shell --nested --wayland
```