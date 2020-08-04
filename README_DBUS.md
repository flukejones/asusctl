# DBUS Guide

```rust
pub static DBUS_NAME: &str = "org.asuslinux.Daemon";
pub static DBUS_PATH: &str = "/org/asuslinux/Daemon";
pub static DBUS_IFACE: &str = "org.asuslinux.Daemon";
```

## Methods

- `SetKeyBacklight`
- `GetKeyBacklight`
- `AnimatrixWrite`
- `SetFanMode`
- `GetFanMode`
- `SetChargeLimit`
- `GetChargeLimit`
- `GetKeyBacklightModes`

## Signals

- `KeyBacklightChanged`
- `FanModeChanged`
- `ChargeLimitChanged`

## Method Inputs

### SetKeyBacklight

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

### GetKeyBacklight

This method will return a JSON string in the same format as accepted by `SetKeyBacklight`.

### GetKeyBacklightModes

Will return a JSON string array of modes that this laptop accepts. The mode data
within this will be the current settings per mode. Good for:

- Getting supported modes
- Getting all mode settings

### AnimatrixWrite

Used to write data to the AniMe display if available. Currently is takes `[[u8; 640]; 2]`
which must be the byte data that will be written directly to the USB device.

### SetFanMode

Accepts an integer from the following:

- `0`: Normal
- `1`: Boost mode
- `2`: Silent mode

### GetFanMode

Returns the integer set from above.

### SetChargeLimit

Accepts an integer in the range of 20-100.

### GetChargeLimit

Returns the integer set from above.

## Signal Outs

### KeyBacklightChanged

When emitted, it will emit the JSON data of the mode changed to, e.g:

```
{
  "Static": {
    "colour": [ 255, 0, 0]
  }
}
```

### FanModeChanged

When emitted, it will include the integer the fan mode was changed to.

### ChargeLimitChanged

When emitted, it will include the integer the charging limit was changed to.

## dbus-send examples

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
dbus-send --system --print-reply --dest=org.asuslinux.Daemon /org/asuslinux/Daemon org.freedesktop.DBus.Introspectable.Introspect > xml/dbus-0.14.4.xml
```