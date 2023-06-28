# asusctl ([-gex]: Gnome extension) -inactive(until v5.0.0)-

## inactive (kind of - update)

~~This project is currently inactive until a new maintainer wants to put some love into it and make it compatible with the newest asusctl versions.~~

ROG Control Center will also have it's own appindicator.

This extension is currently marked as inactive also on extensions.gnome.org and for users not visible anymore.

*UPDATE:*

The origin maintainer (ZaPpPeL) is back from his rabbit-hole! (I'll take care of getting this into a working-state again within the next weeks - stay tuned!)

---

Extension for visualizing [asusctl](https://gitlab.com/asus-linux/asusctl)(`asusd`) settings and status.

*hint:* supergfxctl GPU mode switching moved to another extension to make it platform independent: [supergfxctl-gex](https://gitlab.com/asus-linux/supergfxctl-gex)(`supergfxctl-gex`)

---

## Table of contents

[[_TOC_]]

---

## Extension Features

* Notifications:
  * Battery Charge Limit
* Popup Menu with options to:
  * if supported by laptop model:
    * change the battery charging limit
    * change AniMe Matrix brightness
    * enable / disable AniMe Matrix
* Extension Settings:
  * Enable / disable notifications
  * Enable debug message logging

### Waiting for implementation:

* Configuration interface (prefs)
  * bind ROG-Button to open prefs (if not `rog-control-center` is used)
  * create canvas based fan-curve editing

---

## Icons/Screenshots

_The screenshots below are just examples and might not represent the current used icons._

### Screenshot

![screenshot.png](https://gitlab.com/asus-linux/asusctl-gex/-/raw/main/screenshots/screenshot.png)

**battery charge limit notification:**

![notification.png](https://gitlab.com/asus-linux/asusctl-gex/-/raw/main/screenshots/notification.png)

---

## Requirements

* gnome >= 3.36.0
* [asusctl](https://gitlab.com/asus-linux/asusctl) >= 4.0

---

## Build Instructions

### Dependencies

* nodejs >= 14.0.0
* npm >= 6.14.0

### Building (production)

In a terminal enter the following commands as a user (**do NOT run as root or sudo**):

```bash
git clone https://gitlab.com/asus-linux/asusctl-gex.git /tmp/asusctl-gex && cd /tmp/asusctl-gex
npm install
npm run build && npm run install-user
```

_HINT: You will need to reload the GNOME Shell afterwards. (`Alt + F2` -> `r` on X11, `logout` on Wayland)_

### Building (development)

Instead of the
`npm run build && npm run install-user`
above, use this line instead:
`npm run build && npm run install-dev`

This will remove any production versions and installs the development version instead.

_HINT: You will need to reload the GNOME Shell afterwards. (`Alt + F2` -> `r` on X11, `logout` on Wayland)_ and probably manually enable the extension again.

### Source debugging

`cd` into the directory where you've downloaded the `asusctl-gex` source code and enter the following commands:

```bash
npm install
npm run debug
```

---

## License & Trademarks

**License:** Mozilla Public License Version 2.0 (MPL-2)

**Trademarks:** ASUS and ROG Trademark is either a US registered trademark or trademark of ASUSTeK Computer Inc. in the United States and/or other countries.
Reference to any ASUS products, services, processes, or other information and/or use of ASUS Trademarks does not constitute or imply endorsement, sponsorship, or recommendation thereof by ASUS.
The use of ROG and ASUS trademarks within this website and associated tools and libraries is only to provide a recognisable identifier to users to enable them to associate that these tools will work with ASUS ROG laptops.
