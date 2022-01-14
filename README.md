# `asusctl` for ASUS ROG

[![](https://www.paypalobjects.com/en_US/i/btn/btn_donate_LG.gif)](https://www.paypal.com/donate/?hosted_button_id=4V2DEPS7K6APC) - [Asus Linux Website](https://asus-linux.org/)

`asusd` is a utility for Linux to control many aspects of various ASUS laptops
but can also be used with non-asus laptops with reduced features.

## Kernel support

**The minimum supported kernel version is 5.15**

Fan curve control on laptops with this feature require [this patch](https://lkml.org/lkml/2021/10/23/250) which has been merged for 5.17 upstream.

## Goals

1. To provide an interface for rootless control of some system functions most users wish to control such as fan speeds, keyboard LEDs, graphics modes.
2. Enable third-party apps to use the above with dbus methods
3. To make the above as easy as possible for new users
4. Respect the users resources: be small, light, and fast

Point 3 means that the list of supported distros is very narrow - fedora is explicitly
supported, while Ubuntu and openSUSE are level-2 support. All other distros are *not*
supported (while asusd might still run fine on them). For best support use fedora 32+ Workstation.

Point 4? asusd currently uses a tiny fraction of cpu time, and less than 1Mb of ram, the way
a system-level daemon should.

## Discord

[Discord server link](https://discord.gg/4ZKGd7Un5t)

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
- [X] AniMatrix display on G14 models that include it
- [X] Set battery charge limit (with kernel supporting this)
- [X] Fan curve control on G14 + G15. Requires kernel patch (should reach 5.15 kernel)
- [X] Toggle bios setting for boot/POST sound
- [X] Toggle bios setting for "dedicated gfx" mode on supported laptops (g-sync)

# BUILDING

Requirements are rust >= 1.57 installed from rustup.io if the distro provided version is too old, and `make`.

**Ubuntu (unsuported):**  
    `apt install libclang-dev libudev-dev`
    `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
    `make`
    `sudo make install`

**fedora:**  
    `dnf install clang-devel systemd-devel`
    `make`  
    `sudo make install`  

## Installing
- Fedora copr = https://copr.fedorainfracloud.org/coprs/lukenukem/asus-linux/
- openSUSE = https://download.opensuse.org/repositories/home:/luke_nukem:/asus/
- Ubuntu = not supported due to packaging woes, but you can build and install on your own.

=======

The default init method is to use the udev rule, this ensures that the service is
started when the device is initialised and ready.

If you are upgrading from a previous installed version, you will need to restart the service or reboot.

```
$ systemctl daemon-reload && systemctl restart asusd
```

You may also need to activate the service for debian install. If running Pop!_OS, I suggest disabling `system76-power` gnome-shell extension and systemd service.

## Uninstalling

Run `sudo make uninstall` in the source repo, and remove `/etc/asusd/`.

# OTHER

## Supporting more laptops

Please file a support request.

# License & Trademarks

Mozilla Public License 2 (MPL-2.0)

---

ASUS and ROG Trademark is either a US registered trademark or trademark of ASUSTeK Computer Inc. in the United States and/or other countries.

Reference to any ASUS products, services, processes, or other information and/or use of ASUS Trademarks does not constitute or imply endorsement, sponsorship, or recommendation thereof by ASUS.

The use of ROG and ASUS trademarks within this website and associated tools and libraries is only to provide a recognisable identifier to users to enable them to associate that these tools will work with ASUS ROG laptops.

---
