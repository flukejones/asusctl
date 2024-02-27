// TODO: macro
    // let conn = zbus::blocking::Connection::system().unwrap();
    // let proxy = PlatformProxyBlocking::new(&conn).unwrap();
    // //
    // let proxy2 = proxy.clone();
    // let handle = ui.as_weak();
    // ui.global::<SystemPageData>().on_set_charge_limit(move |limit| {
    //     if let Some(handle) = handle.upgrade() {
    //         match proxy2.set_charge_control_end_threshold(limit as u8) {
    //             Ok(_) => handle
    //                 .invoke_show_toast(format!("Charge limit successfully set to {limit}").into()),
    //             Err(e) => handle.invoke_show_toast(format!("Charge limit failed: {e}").into()),
    //         }
    //     }
    // });

    // let proxy2 = proxy.clone();
    // let handle = ui.as_weak();
    // ui.global::<SystemPageData>().on_set_panel_od(move |od| {
    //     if let Some(handle) = handle.upgrade() {
    //         match proxy2.set_panel_od(od) {
    //             Ok(_) => handle
    //                 .invoke_show_toast(format!("Panel Overdrive successfully set to {od}").into()),
    //             Err(e) => handle.invoke_show_toast(format!("Panel Overdrive failed: {e}").into()),
    //         }
    //     }
    // });

    // or
    // let handle = ui.as_weak();
    // ui.global::<SystemPageData>().on_applied(move || {
    //     handle
    //         .upgrade_in_event_loop(|handle| {
    //             let data = handle.global::<SystemPageData>();
    //             let charge_changed = data.get_charge_limit() as i32 !=
    // data.get_last_charge_limit();             let charge =
    // data.get_charge_limit() as u8;             tokio::spawn(async move {
    //                 let conn = zbus::Connection::system().await.unwrap();
    //                 let proxy = PlatformProxy::new(&conn).await.unwrap();
    //                 if charge_changed {
    //                     proxy
    //                         .set_charge_control_end_threshold(charge)
    //                         .await
    //                         .unwrap();
    //                 }
    //             });
    //         })
    //         .ok();
    // });