# ASUS NB Ctrl

[![](https://www.paypalobjects.com/en_US/i/btn/btn_donate_LG.gif)](https://www.paypal.com/donate/?hosted_button_id=4V2DEPS7K6APC) - [Asus Linux Website](https://asus-linux.org/)

`asusd` is a utility for Linux to control many aspects of various ASUS laptops
but can also be used with non-asus laptops with reduced features.

## Goals

1. To provide an interface for rootless control of some system functions most users wish to control such as fan speeds, keyboard LEDs, graphics modes.
2. Enable third-party apps to use the above with dbus methods
3. To make the above as easy as possible for new users

Point 3 means that the list of supported distros is very narrow - fedora is explicitly
supported, while Ubuntu and openSUSE are level-2 support. All other distros are *not*
supported (while asusd might still run fine on them). For best support use fedora 32+ Workstation.

**NOTICE:**
1. The following is *not* required for 5.11 kernel versions, as this version includes all the required patches.
2. 2021 hardware has a new keyboard prod_id and the patch is included in 5.12+

'hid-asus-rog` DKMS module from [here](https://download.opensuse.org/repositories/home:/luke_nukem:/asus/).

The module enables the following in kernel:

- Initialising the keyboard
- All hotkeys (FN+Key combos)

## Discord

[Discord server link](https://discord.gg/ngbdKabAnP)

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
- [X] Fancy LED modes (See examples) (currently being reworked)
- [X] Saving settings for reload
- [X] Logging - required for journalctl
- [X] AniMatrix display on G14 models that include it (currently being reworked)
- [X] Set battery charge limit (with kernel supporting this)
- [X] Fan curve control on G14 + G15 thanks to @Yarn1
- [X] Graphics mode switching between iGPU, dGPU, on-demand, and vfio (for VM pass-through)
  + [X] Requires only a logout/login
- [X] Toggle bios setting for boot/POST sound
- [X] Toggle bios setting for "dedicated gfx" mode on supported laptops (g-sync)

# FUNCTIONS

## Graphics switching

`asusd` can switch graphics modes between:
- `integrated`, uses the iGPU only and force-disables the dGPU
- `hybrid`, enables Nvidia prime-offload mode
- `nvidia`, uses the Nvidia gpu only
- `vfio`, binds the Nvidia gpu to vfio for VM pass-through

This can be disabled in the config with `"manage_gfx": false,`. Additionally there
is an extra setting for laptops capable of g-sync dedicated gfx mode to enable the
graphics switching to switch on dedicated gfx for "nvidia" mode.

This switcher conflicts with other gpu switchers like optimus-manager, suse-prime
or ubuntu-prime, system76-power, and bbswitch. If you have issues with `asusd`
always defaulting to `integrated` mode on boot then you will need to check for
stray configs blocking nvidia modules from loading in:
- `/etc/modprobe.d/`
- `/usr/lib/modprope.d/`

**VFIO NOTE:** The vfio modules *must not* be compiled into the kernel, they need
to be separate modules. If you don't plan to use vfio mode then you can ignore this
otherwise you may need a custom built kernel.

To enable vfio switching you need to edit `/etc/asusd/asusd.conf` and change `"gfx_vfio_enable": false,` to true.

### Power management udev rule

If you have installed the Nvidia driver manually you will require the
`data/90-asusd-nvidia-pm.rules` udev rule to be installed in `/etc/udev/rules.d/`.

### fedora and openSUSE

You *may* need a file `/etc/dracut.conf.d/90-nvidia-dracut-G05.conf` installed
to stop dracut including the nvidia modules in the ramdisk if you manually
installed the nvidia drivers.

```
# filename /etc/dracut.conf.d/90-nvidia-dracut-G05.conf
# Omit the nvidia driver from the ramdisk, to avoid needing to regenerate
# the ramdisk on updates, and to ensure the power-management udev rules run
# on module load
omit_drivers+=" nvidia nvidia-drm nvidia-modeset nvidia-uvm "
```

and run `dracut -f` after creating it.

## KEYBOARD BACKLIGHT MODES

Models GA401, GA502, GU502 support LED brightness change only (no RGB).

If you model isn't getting the correct led modes, you can edit the file
`/etc/asusd/asusd-ledmodes.toml`, the LED Mode numbers are as follows:

- Static
- Breathe
- Strobe
- Rainbow
- Star
- Rain
- Highlight
- Laser
- Ripple
- Pulse
- Comet
- Flash

use `cat /sys/class/dmi/id/product_name` to get details about your laptop. You
must restart the `asusd.service` after editing.

# Keybinds

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

# BUILDING

Requirements are rust >= 1.40 installed from rustup.io if the distro provided version is too old, and `make`.

**Ubuntu*:** `apt install libclang-dev libudev-dev`

**fedora:** `dnf install clang-devel systemd-devel`

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

If you would like to run this daemon on another non-ASUS laptop you can. You'll
have all features available except the LED and AniMe control (further controllers
can be added on request). You will need to install the alternative service from
`data/asusd-alt.service`.

## Uninstalling

Run `sudo make uninstall` in the source repo, and remove `/etc/asusd/`.

# USAGE

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

## User NOTIFICATIONS via dbus

If you have a notifications handler set up, or are using KDE or Gnome then you
can enable the user service to get basic notifications when something changes.

```
systemctl --user enable asus-notify.service
systemctl --user start asus-notify.service
```

# OTHER

## AniMe input

You will want to look at what MeuMeu has done with [https://github.com/Meumeu/ZephyrusBling/](https://github.com/Meumeu/ZephyrusBling/)

## Supporting more laptops

Please file a support request.

## Notes:

- If charge limit or fan modes are not working, then you may require a kernel newer than 5.6.10.
- AniMe device check is performed on start, if your device has one it will be detected.
- GA14/GA401 and GA15/GA502/GU502, You will need kernel [patches](https://lab.retarded.farm/zappel/asus-rog-zephyrus-g14/-/tree/master/kernel_patches), these are on their way to the kernel upstream.
- On fedora manually installed Nvidia driver requires a dracut config as follows:
```
# filename/etc/dracut.conf.d/90-nvidia-dracut-G05.conf
# Omit the nvidia driver from the ramdisk, to avoid needing to regenerate
# the ramdisk on updates, and to ensure the power-management udev rules run
# on module load
omit_drivers+=" nvidia nvidia-drm nvidia-modeset nvidia-uvm "
```

# License

Mozilla Public License 2 (MPL-2.0)
