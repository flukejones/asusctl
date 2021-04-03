# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Changed
- Display PNG images on AniMe with scale, position, angle, and brightness
- AniMe display parts split out to individual crate in preparation for publishing
  on crates.io

# [3.3.0] - 2021-04-3
### Changed
- Add ledmodes for G733QS
- Add ledmodes for GA401Q
- Default to vfio disabled in configuration. Will now hard-error if enabled and
  the kernel modules are builtin. To enable vfio switching `"gfx_vfio_enable": false,`
  must be changed to `true` in `/etc/asusd/asusd.conf`

# [3.2.4] - 2021-03-24
### Changed
- Ignore vfio-builtin error if switching to integrated

# [3.2.3] - 2021-03-24
### Changed
- Better handling of session tracking
### Added
- List all profile data
- Get active profile name
- Get active profile data

# [3.2.2] - 2021-03-23
### Changed
- Fix brightness control, again, for non-RGB keyboards

# [3.2.1] - 2021-03-21
### Changed
- Fix brightness control
- Large cleanup of code relating to LED controls

# [3.2.0] - 2021-03-21
### Changed
- Refactor keyboard LED handling
- Added --list for profiles (Thanks @aqez)
- Added --remove for profiles (Thanks @aqez)
- Added a graphics mode: vfio. This attaches Nvidia devices to vfio module.
### Broken
- Per-key LED modes, which need thinking about how to go ahead with for future

# [3.1.7] - 2021-03-11
### Changed
- Refactor many parts of daemon
- Switch out session monitoring to logind-zbus

# [3.1.6] - 2021-03-11
### Changed
- Graphics switching will now wait until all users logged out before switching

### Changed
- Further tweaks to gfx switching
- More logging on gfx switching
- Filter bios help according to supported modes
- Prevent gfx mode switching if in dedicated/G-Sync mode

# [3.1.4] - 2021-03-10
### Changed
- Notify through dbus if user changes profile manually
- Better help on CLI, show help only for supported items
- Bugfix to gfx switcher

# [3.1.3] - 2021-03-10
### Changed
- Hotfix: gracefully handle removing modules in use caused by display-manager not
  fully shutdown at the time of trying to remove modules. It will now retry every
  250ms per module

# [3.1.2] - 2021-03-10
### Changed
- Test and create /etc/X11/xorg.conf.d/ if it doesn't exist
- Hotfix to better report module issues

# [3.1.1] - 2021-03-10
### Changed
- Add missing nvidia module nvidia_uvm to gfx ctrl list

# [3.1.0] - 2021-03-09
### Added
- GU502LU led-modes
### Changed
- Graphics switching is now rebootless, the daemon will now restart the
  display-manager to switch modes instead. Caveats are:
  + There is no confirmation from the daemon, the program issuing the command
    must confirm the request.
  + systemd only
- Laptops with dedicated Nvidia mode:
  + You still must reboot for the bios to switch modes
  + On boot if dedicated mode is active then asusd will update the required configs
    to put display-manager in nvidia mode

# [3.0.0] - 2021-02-22
### Added
- G531GD led modes

# [3.0.0] - 2021-02-14
### Changed
- Write set+apply after each array in multizone
- Remove misc bad logic
- Use same code path as 0x1866 device to configure led support for 0x1854 device
- Remove duplicate code
- Set correct speeds for multizone
- Remove dbus crate in favour of zbus. This removes the external dbus lib requirement.
- Huge internal refactor
- BREAKING CHANGE: Anime code refactor. DBUS method names have changed
- Cleanup fan and cpu control + configs

# [2.2.2] - 2021-01-31
### Changed
- Fix for dedicated gfx capable laptops in integrated mode
- Fix for 0x1854 device

# [2.2.1] - 2021-01-27
### Added
- Add ROG Zephyrus M15 LED config
### Changed
- Bugfixes
- Fix reboot/restartx status for GFX switching
- Update readme
- Change CLI arg tag for fan modes
- Make dracut include the nvidia modules in initramfs

# [2.2.0] - 2021-01-26
### Added
- Dbus command to fetch all supported functions of the laptop. That is, all the
  functions that asusd supports for the currently running laptop.
- Bios setting toggles for:
  + Dedicated gfx toggle (support depends on the laptop)
  + Bios boot POST sound toggle
### Changed
- added config option for dedicated gfx mode on laptops with it to enable
  switching directly to dedicated using `asusctl graphics -m nvidia`

# [2.1.2] - 2021-01-10
### Changed
- Adjust gfx controller to assume that the graphics driver is loaded if the
  mode is set for nvidia/hybrid

# [2.1.1] - 2021-01-09
### Changed
- Updates to dependencies

# [2.1.0] - 2020-10-25
### Added
- Option to turn off AniMe display (@asere)
### Changed
- Change option -k to show current LED bright (@asere)
- Correctly disable GFX control via config
- Panic and exit if config can't be parsed
- Add DBUS method to toggle to next fan/thermal profile
- Add DBUS method to toggle to next/prev Aura mode

# [2.0.5] - 2020-09-29
### Changed
- Bugfixes

# [2.0.4] - 2020-09-24
### Changed
- Better and more verbose error handling and logging in many places.
- Fix timeout for client waiting on reply for graphics switching

# [2.0.2] - 2020-09-21
### Changed
- graphics options via CLI are now a command block:
  + `asusctl graphics`
  + -m Mode <nvidia, hybrid, compute, integrated>
  + -g Get current mode
  + -f Force reboot or restart display manager without confirmation

# [2.0.0] - 2020-09-21
### Changed
- Code refactor to spawn less tasks. Main loop will run only as fast as
  it receives events
- No-longer using tokio or async, reducing resource use
### Added
- A basic user daemon has been added for user notifications over dbus (XDG spec)
- Added a user systemd service for notifications (asus-notify)
- Graphics mode handling <iGPU only, dGPU only, or hybrid>, see asusctl --help
### BREAKING CHANGES
- asusd.conf has changed slightly and will overwrite old configs
- All DBUS methods/signals/paths etc, are all updated and changed

# [1.1.2] - 2020-09-10
### Changed
- Bump rog-fan-curve to new versiont o support GA401IV

# [1.1.1] - 2020-09-10
### Changed
- Correction to AMD turbo setting

# [1.1.0] - 2020-09-10
### Changed
- Uses string instead of debug print for some errors
- Add interface num arg for LED controller (should help support
    older laptops better)
- Some slightly better error messages
- Fix an idiotic mistake in `for i in 0..2.. if i > 0` -_-
- Remove "unsupported" warning on laptop ctrl
- Silence warning about AniMe not existing
- Adjust the turbo-toggle CLI arg
- Version bump for new release with fancurves

## [1.0.2] - 2020-08-13
### Changed
- Bugfixes to led brightness watcher
- Bufixes to await/async tasks

## [1.0.1] - 2020-08-13

- Fix small deadlock with awaits

## [1.0.0] - 2020-08-13

- Major fork and refactor to use asus-hid patch for ASUS N-Key device
