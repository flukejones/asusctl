# asusctrl manual

`asusd` is a utility for Linux to control many aspects of various ASUS laptops
but can also be used with non-asus laptops with reduced features.

## Programs Available

- `asusd`: The main system daemon. It is autostarted by a udev rule and systemd unit.
- `asusd-user`: The user level daemon. Currently will run an anime sequence, with RGB keyboard sequences soon.
- `asusctl`: The CLI for interacting with the system daemon
- `asus-notify`: A notification daemon with a user systemd unit that can be enabled.

## `asusd`

`asusd` is the main system-level daemon which will control/load/save various settings in a safe way for the user, along with exposing a *safe* dbus interface for these interactions. This section covers only the daemon plus the various configuration file options.

The functionality that `asusd` exposes is:

- graphics switching
- anime control
- led keyboard control (aura)
- charge limiting
- bios/efivar control
- profiles (fan/cpu)

each of these will be detailed in sections.

### Graphics switching

`asusd` can switch graphics modes between:
- `integrated`, uses the iGPU only and force-disables the dGPU
- `compute`, enables Nvidia without Xorg. Useful for ML/Cuda
- `hybrid`, enables Nvidia prime-offload mode
- `nvidia`, uses the Nvidia gpu only
- `vfio`, binds the Nvidia gpu to vfio for VM pass-through

Switching to/from Hybrid and Nvidia modes requires a logout only (no reboot). Switching between integrated/compute/vfio does not require a logout and is instant.

#### Required actions in distro

**Rebootless note:** You must edit `/etc/default/grub` to remove `nvidia-drm.modeset=1`
from the line `GRUB_CMDLINE_LINUX=` and then recreate your grub config. In fedora
you can do this with `sudo grub2-mkconfig -o /etc/grub2.cfg` - other distro may be
similar but with a different config location. It's possible that graphics driver updates
may change this.

This switcher conflicts with other gpu switchers like optimus-manager, suse-prime
or ubuntu-prime, system76-power, and bbswitch. If you have issues with `asusd`
always defaulting to `integrated` mode on boot then you will need to check for
stray configs blocking nvidia modules from loading in:
- `/etc/modprobe.d/`
- `/usr/lib/modprope.d/`

#### Config options

1. `"gfx_mode": "<MODE>",`: MODE can be <Integrated, Hybrid, Compute, Nvidia, vfio>
2. `"gfx_last_mode": "Nvidia",`: currently unused
3. `"gfx_managed": true,`: enable or disable graphics switching controller
4. `"gfx_vfio_enable": false,`: enable vfio switching for Nvidia GPU passthrough
5. `"gfx_save_compute_vfio": false,`: wether or not to save the vfio state (so it sticks between boots)

#### Graphics switching notes

**G-Sync note:** Some laptops are capable of using the dGPU as the sole GPU in the system which is generally to enable g-sync on the laptop display panel. This is controlled by the bios/efivar control and will be covered in that section.

**vfio note:** The vfio modules *must not* be compiled into the kernel, they need
to be separate modules. If you don't plan to use vfio mode then you can ignore this
otherwise you may need a custom built kernel.

### AniMe control

Controller for the fancy AniMe matrix display on the lid of some machines. This controller is a work in progress.

#### Config options

If you have an AniMe device a few system-level config options are enabled for you in `/etc/asusd/anime.conf`;

1. `"system": [],`: currently unused, is intended to be a default continuous sequence in future versions
2. `"boot": [],`: a sequence that plays on system boot (when asusd is loaded)
3. `"wake": [],`: a sequence that plays when waking from suspend
4. `"shutdown": [],`: a sequence that plays when shutdown begins
5. `"brightness": <FLOAT>`: global brightness control, where `<FLOAT> is 0.0-1.0

Some default examples are provided but are minimal. The full range of configuration options will be covered in another section of this manual.

### Led keyboard control

The LED controller (e.g, aura) enables setting many of the factory modes available if a laptop supports them. It also enables per-key RGB settings but this is a WIP and will likely be similar to how AniMe sequences can be created.

#### Supported laptops

Models GA401, GA502, GU502 support LED brightness change only (no RGB). However the GA401Q model can actually use three modes; static, breathe, and pulse, plus also use red to control the LED brightness intensity.

All models that have any form of LED mode control need to be enabled via the config file at `/etc/asusd/asusd-ledmodes.toml`. Unfortunately ASUS doesn't provide any easy way to find all the supported modes for all laptops (not even through Armory Crate and its various files, that progrma downloads only the required settings for the laptop it runs on) so each model must be added as needed.

#### Config options

The defaults are located at `/etc/asusd/asusd-ledmodes.toml`, and on `asusd` start it creates `/etc/asusd/aura.conf` whcih stores the per-mode settings. If you edit the defaults file you must remove `/etc/asusd/aura.conf` and restart `asusd.service` with `systemctl restart asusd`.

##### /etc/asusd/asusd-ledmodes.toml

Example:
```toml
[[led_data]]
prod_family = "ROG Zephyrus M15"
board_names = ["GU502LU"]
standard = ["Static", "Breathe", "Strobe", "Pulse"]
multizone = false
per_key = false
```

1. `prod_family`: you can find this in `journalctl -b -u asusd`, or `cat /sys/class/dmi/id/product_name`. It should be copied as written. There can be multiple `led-data` groups of the same `prod_family` with differing `board_names`.
2. `board_names`: is an array of board names in this product family. Find this in the journal as above or by `cat /sys/class/dmi/id/board_name`.
3. `standard` are the factory preset modes, the names should corrospond to Armory Crate names
4. `multizone`: some keyboards have 4 zones of LED control, this enables setting a colour in each zone. The keyboard must support this or it has no effect.
5. `per_key`: enable per-key RGB effects. The keyboard must support this or it has no effect.

##### /etc/asusd/aura.conf

This file can be manually edited if desired, but the `asusctl` CLI tool, or dbus methods are the preferred method. Any manual changes to this file mean that the `asusd.service` will need to be restarted, or you need to cycle between modes to force a reload.

### Charge control

Almost all modern ASUS laptops have charging limit control now. This can be controlled in `/etc/asusd/asusd.conf`.

```json
"bat_charge_limit": 80,
```
where the number is a percentage.

### Bios control

Some options that you find in Armory Crate are available under this controller, so far there is:

- POST sound: this is the sound you here on bios boot post
- G-Sync: this controls if the dGPU (Nvidia) is the *only* GPU, making it the main GPU and disabling the iGPU

These options are not written to the config file as they are stored in efivars. The only way to change these is to use the exposed safe dbus methods, or use the `asusctl` CLI tool.

### Profiles

Profiles provide a method setting up various basic CPU and fan settings in profile blocks which can then be switched between or cycled through. The CPU controls so far are:

- Min/Max percentage of CPU frequency (Intel only for now)
- CPU turbo boost enable or disable
- Fan presets. These are 0: Normal, 1: Boost, 2: Silent.
- Fan curves, override fan-preset. AMD only.

#### Config options

Example:
```json
  "toggle_profiles": [
    "normal",
    "boost",
    "silent"
  ],
  "power_profiles": {
    "boost": {
      "min_percentage": 0,
      "max_percentage": 100,
      "turbo": true,
      "fan_preset": 1,
      "fan_curve": null
    },
    "normal": {
      "min_percentage": 0,
      "max_percentage": 100,
      "turbo": true,
      "fan_preset": 0,
      "fan_curve": null
    },
    "silent": {
      "min_percentage": 0,
      "max_percentage": 100,
      "turbo": true,
      "fan_preset": 2,
      "fan_curve": null
    }
  }
```

1. `"toggle_profiles": [],`: these are the profile names that will be cycled through when using a provided next/prev dbus method.
2. `"power_profiles": {}`: all the available profiles.

#### Fan curves

**fan_curve note:** This is a WIP. Currently it relies on `acpi_call` kernel module which is ancient and hacky, not intended for this purpose. A proper kernel driver is in progress.

See [this document](https://github.com/cronosun/atrofac/blob/master/ADVANCED.md#limits) for details on the string format required, e.g, `"fan_curve": "30c:0%,40c:5%,50c:10%,60c:20%,70c:35%,80c:55%,90c:65%,100c:65%"`.

### Support controller

There is one more controller; the support controller. The sole pupose of this controller is to querie all the other controllers for information about their support level for the host laptop. Returns a json string.

## asusd-user

`asusd-user` is a usermode daemon. The intended purpose is to provide a method for users to run there own custom per-key keyboard effects and modes, AniMe sequences, and possibly their own profiles - all without overwriting the *base* system config. As such some parts of the system daemon will migrate to the user daemon over time with the expectation that the Linux system runs both.

As of now only AniMe is active in this with configuration in `~/.config/rog/`. On first run defaults are created that are intended to work as examples.

The main config is `~/.config/rog/rog-user.cfg`

#### Config options: AniMe

`~/.config/rog/rog-user.cfg` contains a setting `"active_anime": "<FILENAME>"` where `<FILENAME>` is the name of the AniMe config to use, located in the same directory and without the file postfix, e.g, `"active_anime": "anime-doom"`

An AniMe config itself is a file with contents:

```json
{
  "name": "<FILENAME>",
  "anime": []
}
```

`<FILENAME>` is used as a reference internally. `"anime": []` is an array of sequences (WIP).

##### "anime" array options

Each object in the array can be one of:

1. AsusAnimation
2. ImageAnimation
3. Image
4. Pause

##### AsusAnimation

`AsusAnimation` is specifically for running the gif files that Armory Crate comes with. `asusctl` includes all of these in `/usr/share/asusd/anime/asus/`
```json
      "AsusAnimation": {
        "file": "<FILE_PATH>",
        "time": <TIME>,
        "brightness": <FLOAT>
      }
```

##### ImageAnimation

`ImageAnimation` can play *any* gif of any size.

```json
      "ImageAnimation": {
        "file": "<FILE_PATH>",
        "scale": <FLOAT>,
        "angle": <FLOAT>,
        "translation": [
          <FLOAT>,
          <FLOAT>
        ],
        "time": <TIME>,
        "brightness": <FLOAT>
      }
    },
```

##### Image

`Image` currently requires 8bit greyscale png. It will be able to use most in future.

```json
    {
      "Image": {
        "file": "<FILE_PATH>",
        "scale": <FLOAT>,
        "angle": <FLOAT>,
        "translation": [
          <FLOAT>,
          <FLOAT>
        ],
        "time": <TIME>,
        "brightness": <FLOAT>
      }
    },
```

##### Pause

A `Pause` is handy for after an `Image` to hold the `Image` on the AniMe for a period.

```json
    {
      "Pause": {
        "secs": <INT>,
        "nanos": <INT>
      }
    },
```

##### Options for objects

**<FILE_PATH>**

Must be full path: `"/usr/share/asusd/anime/asus/gaming/Controller.gif"` or `/home/luke/Downloads/random.gif`.

**<FLOAT>**

A number from 0.0-1.0.
- `brightness`: If it is brightness it is combined with the system daemon global brightness
- `scale`: 1.0 is the original size with lower number shrinking, larger growing
- `angle`: Rotation angle in radians
- `translation`: Shift the image X -/+, and y -/+

**<TIME>**

Time is the length of time to run the gif for:
```json
        "time": {
          "Time": {
            "secs": 5,
            "nanos": 0
          }
        },
```
A cycle is how many gif loops to run:
```json
        "time": {
          "Cycles": 2
        },
```
`Infinite` means that this gif will never end: 
```json
        "time": "Infinite",
```
`Fade` allows an image or gif to fade in and out, and remain at max brightness to n time:
```json
        "time": {
          "Fade": {
            "fade_in": {
              "secs": 2,
              "nanos": 0
            },
            "show_for": {
              "secs": 1,
              "nanos": 0
            },
            "fade_out": {
              "secs": 2,
              "nanos": 0
            }
          }
        },
```
`show_for` can be `null`, if it is `null` then the `show_for` becomes `gif_time_length - fade_in - fade_out`.
This is period for which the gif or image will be max brightness (as set).

**<INT>**

A plain non-float integer.

## asusctl

`asusctl` is a commandline interface which intends to be the main method of interacting with `asusd`. I can be used in any place a terminal app can be used.

This program will query `asusd` for the `Support` level of the laptop and show or hide options according to this support level.

Most commands are self-explanatory.

### CLI Usage and help

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

### Keybinds

To switch to next/previous Aura modes you will need to bind both the aura keys (if available) to one of:
**Next**
```
asusctl led-mode -n
```
**Previous**
```
asusctl led-mode -p
```

To switch Fan/Thermal profiles you need to bind the Fn+F5 key to `asusctl profile -n`.

## User NOTIFICATIONS via dbus

If you have a notifications handler set up, or are using KDE or Gnome then you
can enable the user service to get basic notifications when something changes.

```
systemctl --user enable asus-notify.service
systemctl --user start asus-notify.service
```

# License & Trademarks

Mozilla Public License 2 (MPL-2.0)

---

ASUS and ROG Trademark is either a US registered trademark or trademark of ASUSTeK Computer Inc. in the United States and/or other countries.

Reference to any ASUS products, services, processes, or other information and/or use of ASUS Trademarks does not constitute or imply endorsement, sponsorship, or recommendation thereof by ASUS.

The use of ROG and ASUS trademarks within this website and associated tools and libraries is only to provide a recognisable identifier to users to enable them to associate that these tools will work with ASUS ROG laptops.

---