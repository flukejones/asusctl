# ASUS NB Ctrl

**NOTICE:**

This program requires the kernel patch [here](https://www.spinics.net/lists/linux-input/msg68977.html) to be applied.
Alternatively you may use the dkms module for 'hid-asus-rog` from one of the
repositories [here](https://download.opensuse.org/repositories/home:/luke_nukem:/asus/).

The patch enables the following in kernel:

- All hotkeys (FN+Key combos)
- Control of keyboard brightness using FN+Key combos (not RGB)
- FN+F5 (fan) to toggle fan modes

You will not get RGB control in kernel (yet), and asusd is still required to
change modes and RGB settings. The previous version of this program is named
`rog-core` and takes full control of the interfaces required - if you can't
apply the kernel patches then `rog-core` is still highly usable.

Many other patches for these laptops, AMD and Intel based, are working their way
in to the kernel.

---
asusd is a utility for Linux to control many aspects of various ASUS laptops.

## Discord

[Discord server link](https://discord.gg/PVyFzWj)

## SUPPORTED LAPTOPS

Most ASUS gaming laptops that have a USB keyboard. If `lsusb` shows something similar
to this:

```
Bus 001 Device 002: ID 0b05:1866 ASUSTek Computer, Inc. N-KEY Device
```

then it may work without tweaks. Technically all other functions except the LED
and AniMe parts should work regardless of your latop make. Eventually this project
will probably suffer another rename once it becomes generic enough to do so.

## Implemented

- [X] System daemon
- [X] User notifications daemon
- [X] Setting/modifying built-in LED modes
- [X] Per-key LED setting
- [X] Fancy LED modes (See examples)
- [X] Saving settings for reload
- [X] Logging - required for journalctl
- [X] AniMatrix display on G14 models that include it
- [X] Set battery charge limit (with kernel supporting this)
- [X] Fancy fan control on G14 + G15 thanks to @Yarn1
- [X] Graphics mode switching between iGPU, dGPU, and On-Demand

## FUNCTIONS

### Graphics switching

A new feature has been added to enable switching graphics modes. This can be disabled
in the config with `"manage_gfx": false,`. Please be aware it is a work in progress.

The CLI option for this does not require root until it asks for it, and provides
instructions.

### KEYBOARD BACKLIGHT MODES

Models GA401, GA502, GU502 support LED brightness change only (no RGB).

If you model isn't getting the correct led modes, you can edit the file
`/etc/asusd/asusd-ledmodes.toml`, the LED Mode numbers are as follows:

```
0   STATIC
1   BREATHING
2   STROBE
3   RAINBOW
4   STAR
5   RAIN
6   HIGHLIGHT
7   LASER
8   RIPPLE
10  PULSE
11  COMET
12  FLASH
13  MULTISTATIC
255 PER_KEY
```

use `cat /sys/class/dmi/id/product_name` to get details about your laptop.

## Requirements for compiling

- `rustc` + `cargo` + `make`
- `libusb-1.0-0-dev`
- `libdbus-1-dev`
- `llvm`
- `libclang-dev`
- `libudev-dev`

## Installing

Packaging and auto-builds are available [here](https://build.opensuse.org/package/show/home:luke_nukem:asus/asus-nb-ctrl)

Download repositories are available [here](https://download.opensuse.org/repositories/home:/luke_nukem:/asus/)

---

Run `make` then `sudo make install` then reboot.

The default init method is to use the udev rule, this ensures that the service is
started when the device is initialised and ready.

If you are upgrading from a previous installed version, you will need to restart the service or reboot.

```
$ systemctl daemon-reload && systemctl restart asusd
```

You may also need to activate the service for debian install. If running Pop!_OS, I suggest disabling `system76-power` gnome-shell extension and systemd service.

## Uninstalling

Run `sudo make uninstall` in the source repo, and remove `/etc/asusd/`.

## Updating

If there has been a config file format change your config will be overwritten. This will
become less of an issue once the feature set is nailed down. Work is happening to enable
parsing of older configs and transferring settings to new.

# Usage

**NOTE! Fan mode toggling requires a newer kernel**. I'm unsure when the patches
required for it got merged - I've tested with the 5.6.6 kernel and above only.
To see if the fan-mode changed cat either:

- `cat /sys/devices/platform/asus-nb-wmi/throttle_thermal_policy` or
- `cat /sys/devices/platform/asus-nb-wmi/fan_boost_mode`

The numbers are 0 = Normal/Balanced, 1 = Boost, 2 = Silent.

Running the program as a daemon manually will require root. Standard (non-daemon)
mode expects to be communicating with the daemon mode over dbus.

Commands are given by:

```
asusctl <option> <command> <command-options>
```

Help is available through:

```
asusctl --help
asusctl <command> --help
```

Some commands may have subcommands:

```
asusctl <command> <subcommand> --help
```

## Daemon mode

If the daemon service is enabled then on boot the following will be reloaded from save:

- LED brightness
- Last used built-in mode
- fan-boost/thermal mode
- battery charging limit

The daemon also saves the settings per mode as the keyboard does not do this
itself - this means cycling through modes with the Aura keys will use the
settings that were used via CLI.

Daemon mode creates a config file at `/etc/asusd/asusd.conf` which you can edit a 
little of. Most parts will be byte arrays, but you can adjust things like
`mode_performance`.

## User daemon for notification via dbus

If you have a notifications handler set up, or are using KDE or Gnome then you
can enable the user service to get basic notifications when something changes.

```
systemctl --user enable asus-notify.service
systemctl --user start asus-notify.service
```
# Other

## DBUS Input

See [README_DBUS.md](./README_DBUS.md).

## AniMe input

You will want to look at what MeuMeu has done with [https://github.com/Meumeu/ZephyrusBling/](https://github.com/Meumeu/ZephyrusBling/)

## Supporting more laptops

Please file a support request.

## Notes:

- If charge limit or fan modes are not working, then you may require a kernel newer than 5.6.10.
- AniMe device check is performed on start, if your device has one it will be detected.
- GA14/GA401 and GA15/GA502/GU502, You will need kernel [patches](https://lab.retarded.farm/zappel/asus-rog-zephyrus-g14/-/tree/master/kernel_patches), these are on their way to the kernel upstream.

# License

Mozilla Public License 2 (MPL-2.0)

# Credits

- [flukejones](https://github.com/flukejones/), project maintainer.
- [tuxuser](https://github.com/tuxuser/)
- [aspann](https://github.com/aspann)
- [meumeu](https://github.com/Meumeu)
- Anyone missed? Please contact me
