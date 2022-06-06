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

The task trait. There are three ways to implement this:

```rust
pub struct CtrlAnimeTask(Arc<Mutex<CtrlAnime>>);

impl crate::CtrlTask for CtrlAnimeTask {
    // This will run once only
    fn create_tasks(&self, executor: &mut Executor) -> Result<(), RogError> {
       if let Ok(lock) = self.inner.try_lock() {
            <some action>
        }
        Ok(())
    }

    // This will run until the notification stream closes (which in most cases will be never)
    fn create_tasks(&self, executor: &mut Executor) -> Result<(), RogError> {
        let connection = Connection::system().await.unwrap();
        let manager = ManagerProxy::new(&connection).await.unwrap();

        let inner = self.inner.clone();
        executor
            .spawn(async move {
                // A notification from logind dbus interface
                if let Ok(p) = manager.receive_prepare_for_sleep().await {
                    // A stream that will continuously output events
                    p.for_each(|_| {
                        if let Ok(lock) = inner.try_lock() {
                            // Do stuff here
                        }
                    })
                    .await;
                }
            })
            .detach();
    }

    // This task will run every 500 milliseconds
    fn create_tasks(&self, executor: &mut Executor) -> Result<(), RogError> {
        let inner = self.inner.clone();
        // This is a provided free trait to help set up a repeating task
        self.repeating_task(500, executor, move || {
            if let Ok(lock) = inner.try_lock() {
                // Do stuff here
            }
        })
        .await;
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

#[async_trait]
impl crate::ZbusAdd for CtrlAnimeZbus {
    fn add_to_server(self, server: &mut zbus::ObjectServer) {
        // This is a provided free helper trait with pre-set body. It will move self in-to.
        Self::add_to_server_helper(self, "/org/asuslinux/Anime", server).await;
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
