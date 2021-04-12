# Daemon

## Controller pattern

There are a series of traits in the daemon for use with controller objects. Not all traits are required:

- `Reloadable`, for controllers that need the ability to reload (typically on start)
- `ZbusAdd`, for controllers that have zbus derive. These need to run on the zbus server.
- `CtrlTask`, for controllers that need to run tasks every loop.
- `GetSupported`, see if the hardware/functions this controller requires are supported.

The first 3 trait objects get owned by the daemon methods that required them, which is why an `Arc<Mutex<T>>` is required.

Generally the actual controller object will need to live in its own world as its own struct.
Then for each trait that is required a new struct is required that can have the trait implemented, and that struct would have a reference to the main controller via `Arc<Mutex<T>>`.

### Example

Main controller:

```rust
pub struct CtrlAnime {
    <things the controller requires>
}

impl CtrlAnime {
    <functions the controller exposes>
}
```

The task trait:

```rust
pub struct CtrlAnimeTask(Arc<Mutex<CtrlAnime>>);

impl crate::CtrlTask for CtrlAnimeTask {
    fn do_task(&self) -> Result<(), RogError> {
       if let Ok(lock) = self.inner.try_lock() {
            <some action>
        }
        Ok(())
    }
}
```

The reloader trait
```rust
pub struct CtrlAnimeReloader(Arc<Mutex<CtrlAnime>>);

impl crate::Reloadable for CtrlAnimeReloader {
    fn reload(&mut self) -> Result<(), RogError> {
        if let Ok(lock) = self.inner.try_lock() {
            <some action>
        }
        Ok(())
    }
}
```

The Zbus requirements:
```rust
pub struct CtrlAnimeZbus(Arc<Mutex<CtrlAnime>>);

impl crate::ZbusAdd for CtrlAnimeZbus {
    fn add_to_server(self, server: &mut zbus::ObjectServer) {
        server
            .at(
                &ObjectPath::from_str_unchecked("/org/asuslinux/Anime"),
                self,
            )
            .map_err(|err| {
                warn!("CtrlAnimeDisplay: add_to_server {}", err);
                err
            })
            .ok();
    }
}

#[dbus_interface(name = "org.asuslinux.Daemon")]
impl CtrlAnimeZbus {
    fn <zbus method>() {
       if let Ok(lock) = self.inner.try_lock() {
            <some action>
        }
    }
}
```

The controller can then be added to the daemon parts as required.
