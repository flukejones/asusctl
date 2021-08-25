use std::{env::args, sync::mpsc::channel};
use supergfxctl::{
    gfx_vendors::{GfxRequiredUserAction, GfxVendors},
    special::{get_asus_gsync_gfx_mode, has_asus_gsync_gfx_mode},
    zbus_proxy::GfxProxy,
};

use gumdrop::Options;
use zbus::Connection;

#[derive(Default, Options)]
struct CliStart {
    #[options(help = "print help message")]
    help: bool,
    #[options(
        meta = "",
        help = "Set graphics mode: <nvidia, hybrid, compute, integrated>"
    )]
    mode: Option<GfxVendors>,
    #[options(help = "Get the current mode")]
    get: bool,
    #[options(help = "Get the current power status")]
    pow: bool,
    #[options(help = "Do not ask for confirmation")]
    force: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = args().skip(1).collect();

    match CliStart::parse_args_default(&args) {
        Ok(command) => {
            do_gfx(command)?;
        }
        Err(err) => {
            eprintln!("source {}", err);
            std::process::exit(2);
        }
    }

    Ok(())
}

fn do_gfx(command: CliStart) -> Result<(), Box<dyn std::error::Error>> {
    if command.mode.is_none() && !command.get && !command.pow && !command.force || command.help {
        println!("{}", command.self_usage());
    }

    let conn = Connection::new_system()?;
    let proxy = GfxProxy::new(&conn)?;

    let (tx, rx) = channel();
    proxy.connect_notify_action(tx)?;

    if let Some(mode) = command.mode {
        if has_asus_gsync_gfx_mode() && get_asus_gsync_gfx_mode()? == 1 {
            println!("You can not change modes until you turn dedicated/G-Sync off and reboot");
            std::process::exit(-1);
        }

        println!("If anything fails check `journalctl -b -u supergfxd`\n");

        proxy.gfx_write_mode(&mode).map_err(|err|{
            println!("Graphics mode change error. You may be in an invalid state.");
            println!("Check mode with `-g` and switch to opposite\nmode to correct it, e.g: if integrated, switch to hybrid, or if nvidia, switch to integrated.\n");
            err
        })?;

        loop {
            proxy.next_signal()?;

            if let Ok(res) = rx.try_recv() {
                match res {
                    GfxRequiredUserAction::Integrated => {
                        println!(
                            "You must change to Integrated before you can change to {}",
                            <&str>::from(mode)
                        );
                    }
                    GfxRequiredUserAction::Logout | GfxRequiredUserAction::Reboot => {
                        println!(
                            "Graphics mode changed to {}. User action required is: {}",
                            <&str>::from(mode),
                            <&str>::from(&res)
                        );
                    }
                    GfxRequiredUserAction::None => {
                        println!("Graphics mode changed to {}", <&str>::from(mode));
                    }
                }
            }
            std::process::exit(0)
        }
    }
    if command.get {
        let res = proxy.gfx_get_mode()?;
        println!("Current graphics mode: {}", <&str>::from(res));
    }
    if command.pow {
        let res = proxy.gfx_get_pwr()?;
        println!("Current power status: {}", <&str>::from(&res));
    }

    Ok(())
}
