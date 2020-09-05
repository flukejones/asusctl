# ASUS NB Ctrl

**NOTICE:**

This program requires the kernel patch in `./kernel-patch/` to be applied.
As of 04/08/2020 these have been submitted to lkml. Alternatively you may
use the dkms module for 'hid-asus-rog` from one of the repositories [here](https://download.opensuse.org/repositories/home:/luke_nukem:/asus/).

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

If your laptop is not in the following lists, it may still work with fan-mode switching and charge limit control.

**Please help test or provide info for:**

- GL703(0x1869)
- GL553/GL753 (device = 0x1854) (attempted support from researching 2nd-hand info, multizone may work)

**Laptop support is modified on a per-case basis** as the EC for the keyboard varies
a little between models, e.g, some RGB modes are missing, or it's a single colour.
As far as I can see, the EC does not give us a way to find what modes are supported.

### ANIME AND OTHER FUNCTIONS

**AniMe device check is performed on start, if your device has one it will be detected.**

**NOTE:** If charge limit or fan modes are not working, then you may require a kernel newer than 5.6.10.

- [X] AniMe Matrix display
- [X] Power profile switching on fan-mode (FN+F5)
  - [X] Intel
    - [X] Turbo enale/disable
    - [X] Min frequency percentage
    - [X] Max frequency percentage
  - [X] AMD
    - [X] Turbo enale/disable
- [X] Battery charge limit

**NOTE:** GA14/GA401 and GA15/GA502/GU502, You will need kernel [patches](https://lab.retarded.farm/zappel/asus-rog-zephyrus-g14/-/tree/master/kernel_patches).

### KEYBOARD BACKLIGHT MODES

Models GA401, GA502, GU502 support LED brightness change only (no RGB).

| MODEL  | STATIC | BREATHING | STROBE | RAINBOW | STAR | RAIN | HIGHLIGHT | LASER | RIPPLE | PULSE | COMET | FLASH | ZONES | PER-KEY RGB |
|:------:|:------:|:---------:|:------:|:-------:|:----:|:----:|:---------:|:-----:|:------:|:-----:|:-----:|:-----:|:-----:|:-----------:|
| G512LI |   X    |     X     |    X   |    X    |      |      |           |       |        |       |       |       |       |             |
| G712LI |   X    |     X     |    X   |    X    |      |      |           |       |        |       |       |       |       |             |
| GM501  |   X    |     X     |    X   |    X    |      |      |           |       |        |       |       |       |   X   |             |
| GX531  |   X    |     X     |    X   |    X    |      |      |           |       |        |       |       |       |   X   |             |
| G512   |   X    |     X     |    X   |    X    |      |      |           |       |        |       |       |       |   X   |             |
| G712   |   X    |     X     |    X   |    X    |      |      |           |       |        |       |       |       |   X   |             |
| GX502  |   X    |     X     |    X   |    X    |  X   |  X   |     X     |   X   |    X   |   X   |   X   |   X   |       |     X       |
| GX701  |   X    |     X     |    X   |    X    |  X   |  X   |     X     |   X   |    X   |   X   |   X   |   X   |       |     X       |
| G531   |   X    |     X     |    X   |    X    |  X   |  X   |     X     |   X   |    X   |   X   |   X   |   X   |   X   |     X       |
| G731   |   X    |     X     |    X   |    X    |  X   |  X   |     X     |   X   |    X   |   X   |   X   |   X   |   X   |     X       |
| G532   |   X    |     X     |    X   |    X    |  X   |  X   |     X     |   X   |    X   |   X   |   X   |   X   |       |     X       |

It is highly likely this doesn't cover all models.

For editing the `/etc/asusd/asusd-ledmodes.toml`, the LED Mode numbers are as follows:

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

## Implemented

- [X] Daemon
- [X] Setting/modifying built-in LED modes
- [X] Per-key LED setting
- [X] Fancy LED modes (See examples)
- [X] Saving settings for reload
- [X] Logging - required for journalctl
- [X] AniMatrix display on G14 models that include it
- [X] Set battery charge limit (with kernel supporting this)

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

You may also need to activate the service for debian install. If running Pop!_OS, I suggest disabling `system76-power`
gnome-shell extension, or at least limiting use of the power-management parts as `asusd` lets you set the same things
(one or the other will overwrite pstates). I will create a shell extension at some point similar to system76, but using
the asusd parts. It is safe to leave `system76-power.service` enabled and use for switching between graphics modes.

## Uninstalling

Run `sudo make uninstall` in the source repo, and remove `/etc/asusd.conf`.

## Updating

Occasionally you need to remove `/etc/asusd.conf` and restart the daemon to create a new one. You *can* back up the old
one and copy settings back over (then restart daemon again).

# Usage

**NOTE! Fan mode toggling requires a newer kernel**. I'm unsure when the patches required for it got merged - I've
tested with the 5.6.6 kernel and above only. To see if the fan-mode changed cat either:

- `cat /sys/devices/platform/asus-nb-wmi/throttle_thermal_policy` or
- `cat /sys/devices/platform/asus-nb-wmi/fan_boost_mode`

The numbers are 0 = Normal/Balanced, 1 = Boost, 2 = Silent.

Running the program as a daemon manually will require root. Standard (non-daemon) mode expects to be communicating with
the daemon mode over dbus.

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

### Example

```
$ asusctl --help
Usage: asusctl [OPTIONS]

Optional arguments:
  -h, --help             print help message
  -v, --version          show program version number
  -k, --kbd-bright VAL   <off, low, med, high>
  -p, --pwr-profile PWR  <silent, normal, boost>
  -c, --chg-limit CHRG   <20-100>

Available commands:
  led-mode  Set the keyboard lighting from built-in modes
  profile   Create and configure profiles

$ asusctl profile --help
Usage: asusctl profile [OPTIONS]

Positional arguments:
  profile

Optional arguments:
  -h, --help         print help message
  -c, --create       create the profile if it doesn't exist
  -t, --turbo        enable cpu turbo (AMD)
  -n, --no-turbo     disable cpu turbo (AMD)
  -m, --min-percentage MIN-PERCENTAGE
                     set min cpu scaling (intel)
  -M, --max-percentage MAX-PERCENTAGE
                     set max cpu scaling (intel)
  -p, --preset PWR   <silent, normal, boost>
  -C, --curve CURVE  set fan curve

$ asusctl led-mode --help
Usage: asusctl led-mode [OPTIONS]

Optional arguments:
  -h, --help  print help message

Available commands:
  static        set a single static colour
  breathe       pulse between one or two colours
  strobe        strobe through all colours
  rainbow       rainbow cycling in one of four directions
  star          rain pattern mimicking raindrops
  rain          rain pattern of three preset colours
  highlight     pressed keys are highlighted to fade
  laser         pressed keys generate horizontal laser
  ripple        pressed keys ripple outwards like a splash
  pulse         set a rapid pulse
  comet         set a vertical line zooming from left
  flash         set a wide vertical line zooming from left
  multi-static  4-zone multi-colour

$ asusctl led-mode static --help
Usage: asusctl led-mode static [OPTIONS]

Optional arguments:
  -h, --help  print help message
  -c HEX      set the RGB value e.g, ff00ff

$ asusctl led-mode star --help
Usage: asusctl led-mode star [OPTIONS]

Optional arguments:
  -h, --help  print help message
  -c HEX      set the first RGB value e.g, ff00ff
  -C HEX      set the second RGB value e.g, ff00ff
  -s SPEED    set the speed: low, med, high
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

Daemon mode creates a config file at `/etc/asusd.conf` which you can edit a 
little of. Most parts will be byte arrays, but you can adjust things like
`mode_performance`.

### DBUS Input

See [README_DBUS.md](./README_DBUS.md).

### AniMe input

You will want to look at what MeuMeu has done with [https://github.com/Meumeu/ZephyrusBling/](https://github.com/Meumeu/ZephyrusBling/)

### Wireshark captures

TODO: see `./wireshark_data/` for some captures.

### Supporting more laptops

Please file a support request.

## License

Mozilla Public License 2 (MPL-2.0)

# Credits

- [flukejones](https://github.com/flukejones/), project maintainer.
- [tuxuser](https://github.com/tuxuser/)
- [aspann](https://github.com/aspann)
- [meumeu](https://github.com/Meumeu)
- Anyone missed? Please contact me
