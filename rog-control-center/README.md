# App template

This is a trial app cut down to bare essentials to show how to create an app that can run in the background
with some user-config options.

egui based. Keep in mind that this is very much a bit of a mess due to experimenting.

## Running

Use `WINIT_UNIX_BACKEND=x11 rog-control-center`. `WINIT_UNIX_BACKEND` is required due to window decorations not updating and the window not really being set as visible/invisible on wayland.

## Build features

For testing some features that are typically not available on all laptops:

```rust
cargo run --features mocking
```

## TODO

- Add notification watch for certain UI elements to enforce an update (for example when a user changes Aura via a hot key).
