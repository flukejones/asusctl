# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased - 4.5.0]
### Added
- intofy watches on:
  - `charge_control_end_threshold`
  - `panel_od`
  - `gpu_mux_mode`
  - `platform_profile`
  - keyboard brightness
  - These allow for updating any associated config and sending dbus notifications.
- New dbus methods
  - `DgpuDisable`
  - `SetDgpuDisable`
  - `NotifyDgpuDisable`
  - `EgpuEnable`
  - `SetEgpuEnable`
  - `NotifyEgpuEnable`
  - `MainsOnline` (This is AC, check if plugged in or not)
  - `NotifyMainsOnline`
### Changed
- Use loops to ensure that mutex is gained for LED changes.
- asusctl now uses tokio for async runtime. This helps simplify some code.
### Breaking
- DBUS: all charge control methods renamed to:
  - `ChargeControlEndThreshold`
  - `SetChargeControlEndThreshold`
  - `NotifyChargeControlEndThreshold`
  - `PanelOd` (form PanelOverdrive)
  - `SetPanelOd`
  - `NotifyPanelOd`
  - Path `/org/asuslinux/Charge` changed to `/org/asuslinux/Power`

## [v4.4.0] - 2022-08-29
### Added
- Support for per-key config has been added to `asusd-user`. At the moment it is
  basic with only a few effects done. Please see the manual for more information.
- Support for unzoned and  per-zone effects on some laptops. As above.
- Added three effects to use with Zoned or Per-Key:
  + Static, Breathe, Flicker. More to come.
- Support for G713RS LED modes
- Support for TUF laptop RGB (kernel patches required, these are submitted upstream)
### Changed
- Create new rog-platform crate to manage all i/o in a universal way
  + kbd-led handling (requires kernel patches, TUF specific)
  + platform handling (asus-nb-wmi)
  + power (basic, can be extended in future)
  + hidraw
  + usbraw
- Refactor how ROGCC handles IPC for background open, run-in-bg
- Refactor daemon task creation to be simpler (for development)
- Rename dpu_only to gpu_mux. Update all related messages and info.
### Breaking
- DBUS: rename path `/org/asuslinux/RogBios` to `/org/asuslinux/Platform`
- DBUS: renamed `dedicated_graphic_mode` to `gpu_mux_mode` (`GpuMuxMode`)
- DBUS: renamed `set_dedicated_graphic_mode` to `set_gpu_mux_mode` (`SetGpuMuxMode`)
  + The methods above take an enum: 0 = Discrete, 1 = Optimus

## [4.3.4] - 2022-08-03
### Bugfix
- ROGCC: Remove power setting from correct array

## [4.3.3] - 2022-08-02
### Added
- `rog-control-center` has now been moved in to the main workspace due to
  the heavy dependencies on most of the rog crates
- Preliminary support of TUF RGB keyboards + power states
- Support for G713RW LED modes (Author: jarvis2709)
- Support for G713IC LED modes
### Changed
- The udev rules have been changed to make asusd load with all gamer variants when asus-nb-wmi is loaded
  - TUF, ROG, Zephyrus, Strix

## [4.3.0] - 2022-07-21
### Added
- Clear command for anime `asusctl anime --clear` will clear the display
- Re-added support for LED power states on `0x1866` type keyboards
### Changed
- Make rog-anime more error tolerent. Remove various asserts and return errors instead
- Return error if a pixel-gif is larger than the anime-display dimensions
- Both Anime and Aura dbus interfaces are changed a little
  - Aura power has changed, all power related settings are now in one method
  - Anime methods will now return an error (if errored)
  - /org/asuslinux/Led renamed to /org/asuslinux/Aura

## [4.2.1] - 2022-07-18
### Added
- Add panel overdrive support (autodetects if supported)
- Add detection of dgpu_disable and egpu_enable for diagnostic
### Changed
- Fixed save and restore of multizone LED settings
- Create defaults for multizone

## [4.2.0] - 2022-07-16
### Added
- Support for GA402 Anime Matrix display (Author: I-Al-Istannen & Luke Jones)
- Support for power-config of all LED zones. See `asusctrl led-power --help` (Author: Luke Jones, With much help from: @MNS26)
- Full support for multizone LED <logo, keyboard, lightbar> (Author: Luke Jones, With much help from: @MNS26)
- Add ability to load extra data from `/etc/asusd/asusd-user-ledmodes.toml` for LED support if file exits
- Support for G513IM LED modes
- Support for GX703HS LED modes
### Changed
- Dbus interface for Aura config has been changed, all power control is done with `SetLedsEnabled` and `SetLedsDisabled`
- Data for anime-matrix now requires passing the laptop model as enum
- Extra unit tests for anime stuff to help verify things

### Added
- Support for GA503R LED modes
### Changed
- Refactor LED and AniMe tasks
- Reload keyboard brightness on resume from sleep/hiber

## [4.1.1] - 2022-06-21
### Changed
- Fixes to anime matrix system thread cancelation

## [4.1.0] - 2022-06-20
### Changed
- Huge refactor to use zbus 2.2 + zvariant 3.0 in system-daemon.
- Daemons with tasks now use `smol` for async ops.
- Fixes to fan-curve settings from CLI (Author: Armas Span)
- Add brightness to anime zbus notification
- Adjust how threads in AniMe matrix controller work
- Use proper power-state packet for keyboard LED's (Author: Martin Piffault)
### Added
- Support for GA402R LED modes
- Support for GU502LV LED modes
- Support for G512 LED modes
- Support for G513IC LED modes (Author: dada513)
- Support for G513QM LED modes (Author: Martin Piffault)
- Add side-LED toggle support (Author: Martin Piffault)
- Support reloading keyboard mode on wake (from sleep/hiber)
- Support reloading charge-level on wake (from sleep/hiber)
- Support running AniMe animation blocks on wake/sleep and boot/shutdown events

# [4.0.7] - 2021-12-19
### Changed
- Fix incorrect power-profile validation
- Update asusd-ledmodes.toml to support Asus Rog Strix G15 G513QE (@LordVicky)
- Update patch notes and links

# [4.0.6] - 2021-11-01
### Changed
- Fix CLI for bios toggles
### Added
- Extra commands for AniMe: pixel-image, gif, pixel-gif

# [4.0.5] - 2021-10-27
### Changed
- Convert fan curve percentage to 0-255 expected by kernel driver only if '%' char is used, otherwise the expected range for fan power is 0-255
- Use correct error in daemon for invalid charging limit
- Enforce charging limit values in range 20-100
### Added
- LED modes for G513QR

# [4.0.4] - 2021-10-02
### Changed
- Add missing Profile commands
- Spawn tasks on individual threads to prevent blocking
- Don't force fan-curve default on reload
- Begin obsoleting the graphics switch command in favour of supergfxctl
- Slim down the notification daemon to pure ASUS notifications

# [4.0.3] - 2021-09-16
### Changed
- Don't show fan-curve warning if fan-curve available
- Add G713QR to Strix led-modes
- Fix part of CLI fan-curve control

# [4.0.2] - 2021-09-14
### Changed
- Backup old configs to *-old if parse fails
- Prevent some types of crashes related to unpatched kernels
- Add better help for graphics errors
- Add better help for asusctl general errors
- Implement fan-curve dbus API
- Implement partial fan-curve control via CLI tool
  + Set fan curve for profile + fan gpu/cpu

# [4.0.1] - 2021-09-11
### Changed
- Fix asusd-ledmodes.toml

# [4.0.0] - 2021-09-10
### Added
- AniMe:
  + Support 8bit RGB, RGBA, 16bit Greyscalw, RGB, RGBA
  + add `AsusImage` type for slanted-template pixel-perfect images
  + `BREAKING:` plain `Image` with time period is changed and old anime configs break as a result (sorry)
- LED:
  + By popular request LED prev/next cycle is added
  + Add led modes for GX551Q
### BREAKING CHANGES
- Graphics control:
  + graphics control is pulled out of asusd and moved to new package; https://gitlab.com/asus-linux/supergfxctl
- Proflies:
  + profiles now depend on power-profile-daemon plus kernel patches for support of platform_profile
    - if your system supports fan-curves you will also require upcoming kernel patches for this
  + profiles are now moved to a new file
  + fan-curves are only partially completed due to this release needing to be done sooner

# [3.7.2] - 2021-08-02
### Added
- Enable multizone support on Strix 513IH
- Add G513QY ledmodes
### Changed
- Fix missing CLI command help for some supported options
- Fix incorrectly selecting profile by name, where the active profile was being copied to the selected profile
- Add `asusd` version back to `asusctl -v` report
- Fix various clippy warnings

# [3.7.1] - 2021-06-11
### Changed
- Refine graphics mode switching:
  + Disallow switching to compute or vfio mode unless existing mode is "Integrated"

# [3.7.0] - 2021-06-06
### Changed
- Set PM to auto for Nvidia always
- Extra info output for gfx dev scan
- Extra info in log for G-Sync to help prevent user confusion around gfx switching
- Add GA503Q led modes
- Added ability to fade in/out gifs and images for anime. This does break anime configs. See manual for details.
- Added task to CtrlLed to set the keyboard LED brightness on wake from suspend
  + requires a kernel patch which will be upstreamed and in fedora rog kernel
- Make gfx change from nvidia to vfio/compute also force-change to integrated _then_
  to requested mode
- Fix invalid gfx status when switching from some modes
- Fix copy over of serde skipped config values on config reload

# [3.6.1] - 2021-05-25
### Changed
- Bugfix: write correct fan modes for profiles
- Bugfix: apply created profiles

# [3.6.1] - 2021-05-25
### Changed
- Bugfix for cycling through profiles

# [3.6.0] - 2021-05-24
### Changed
- Add GX550L led modes
- Don't save compute/vfio modes. Option in config for this is removed.
- Store a temporary non-serialised option in config for if compute/vfio is active
  for informational purposes only (will not apply on boot)
- Save state for LEDs enabled + sleep animation enabled
- Save state for AnimMe enabled + boot animation enabled
- Add extra config options and dbus methods
- Add power state signals for anime and led
- Refactor to use channels for dbus signal handler send/recv
- Split out profiles independant parts to a rog-profiles crate
- Cleanup dependencies
- Fix some dbus Supported issues

# [3.5.2] - 2021-05-15
### Changed
- Bugfix: prevent the hang on compute/integrated mode change

# [3.5.1] - 2021-04-25
### Changed
+ Anime:
  - Fix using multiple configs

# [3.5.0] - 2021-04-25
### Changed
+ Keyboard:
  - Split out all aura functionality that isn't dependent on the daemon in to a
    new crate `rog-aura` (incomplete)
  - Keyboard LED control now includes:
    + Enable/disable LED's while laptop is awake
    + Enable/disable LED animation while laptop is suspended and AC plugged in
  - Properly reload the last used keyboard mode on boot
+ Graphics:
  - Correctly enable compute mode for nvidia plus no-reboot or logout if switching
    from vfio/integrated/compute.
  - Add asusd config option to not save compute/vfio mode switch.
+ Anime:
  - Enable basic multiple user anime configs (asusd-user must still be restarted)
+ Profiles:
  - Enable dbus methods for freq min/max, fan curve, fan preset, CPU turbo enable.
    These options will apply to the active profile if no profile name is specified.

# [3.4.1] - 2021-04-11
### Changed
- Fix anime init sequence

# [3.4.0] - 2021-04-11
### Changed
- Revert zbus to 1.9.1
- Use enum to show power states, and catch missing pci path for nvidia.
- Partial user-daemon for anime/per-key done, `asusd-user`. Includes asusd-user systemd unit.
- user-daemon provides dbus emthods to insert anime actions, remove from index, set leds on/off
  + Config file is stored in `~/.config/rog/rog-user.cfg`
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
