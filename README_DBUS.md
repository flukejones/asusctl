# DBUS Guide

**WARNING: In progress updates**

Interface name = org.asuslinux.Daemon

Paths:
- `/org/asuslinux/Gfx`
  + `SetVendor` (string)
  + `NotifyVendor` (recv vendor label string)
- `/org/asuslinux/Led`
  + `LedMode` (AuraMode as json)
  + `LedModes` (array[AuraMode] as json)
  + `SetLedMode` (AuraMode -> json)
  + `NotifyLed` (recv json data)
- `/org/asuslinux/Anime`
  + `SetAnime` (byte array data)
- `/org/asuslinux/Charge`
  + `Limit` (u8)
  + `SetLimit` (u8)
  + `NotifyCharge` (recv i8)
- `/org/asuslinux/Profile`
  + `Profile` (recv current profile data as json string)
  + `Profiles` (recv profiles data as json string (map))
  + `SetProfile` (event -> json)
  + `NotifyProfile` (recv current profile name)

All `Notify*` methods are signals.

### SetLed

This method expects a string of JSON as input. The JSON is of format such:

```
{
  "Static": {
    "colour": [ 255, 0, 0]
  }
}
```

The possible contents of a mode are:

- `"colour": [u8, u8, u8],`
- `"speed": <String>,` <Low, Med, High>
- `"direction": <String>,` <Up, Down, Left, Right>

Modes may or may not be available for a specific laptop (TODO: dbus getter for
supported modes). Modes are:

- `"Static": { "colour": <colour> },`
- `"Pulse": { "colour": <colour> },`
- `"Comet": { "colour": <colour> },`
- `"Flash": { "colour": <colour> },`
- `"Strobe": { "speed": <speed> },`
- `"Rain": { "speed": <speed> },`
- `"Laser": { "colour": <colour>, "speed": <speed> },`
- `"Ripple": { "colour": <colour>, "speed": <speed> },`
- `"Highlight": { "colour": <colour>, "speed": <speed> },`
- `"Rainbow": { "direction": <direction>, "speed": <speed> },`
- `"Breathe": { "colour": <colour>, "colour2": <colour>, "speed": <speed> },`
- `"Star": { "colour": <colour>, "colour2": <colour>, "speed": <speed> },`
- `"MultiStatic": { "colour1": <colour>, "colour2": <colour>, , "colour3": <colour>, "colour4": <colour> },`

Additionally to the above there is `"RGB": [[u8; 64]; 11]` which is for per-key
setting of LED's but this requires some refactoring to make it easily useable over
dbus.

Lastly, there is `"LedBrightness": <u8>` which accepts 0-3 for off, low, med, high.

### SetFanMode

Accepts an integer from the following:

- `0`: Normal
- `1`: Boost mode
- `2`: Silent mode

## dbus-send examples OUTDATED

```
dbus-send --system --type=method_call --dest=org.asuslinux.Daemon /org/asuslinux/Daemon org.asuslinux.Daemon.SetKeyBacklight string:'{"Static": {"colour": [ 80, 0, 40]}}'
```

```
dbus-send --system --type=method_call --dest=org.asuslinux.Daemon /org/asuslinux/Daemon org.asuslinux.Daemon.SetKeyBacklight string:'{"Star":{"colour":[0,255,255],"colour2":[0,0,0],"speed":"Med"}}'
```

**Note:** setting colour2 to `[0,0,255]` activates random star colour. Colour2 has no effect on the
mode otherwise.
```
dbus-send --system --type=method_call --dest=org.asuslinux.Daemon /org/asuslinux/Daemon org.asuslinux.Daemon.SetKeyBacklight string:'{"Star":{"colour":[0,255,255],"colour2":[0,0,255],"speed":"Med"}}'
```

```
dbus-send --system --type=method_call --dest=org.asuslinux.Daemon /org/asuslinux/Daemon org.asuslinux.Daemon.SetKeyBacklight string:'{"LedBrightness":3}'
```

```
dbus-send --system --type=method_call --dest=org.asuslinux.Daemon /org/asuslinux/Daemon org.asuslinux.Daemon.SetFanMode byte:'2'
```

Monitoring dbus while sending commands via `rog-core` will give you the json structure if you are otherwise unsure, e.g: `dbus-monitor --system |grep -A2 asuslinux`.

## Getting an introspection .xml

```
dbus-send --system --print-reply --dest=org.asuslinux.Daemon /org/asuslinux/Charge org.freedesktop.DBus.Introspectable.Introspect > xml/asusd-charge.xml
```