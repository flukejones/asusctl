mod zbus_logind;

#[cfg(test)]
mod tests {
    use zbus::Connection;

    use crate::zbus_logind;

    #[test]
    fn it_works() {
        let conn = Connection::new_system().unwrap();
        let proxy = zbus_logind::ManagerProxy::new(&conn).unwrap();

        let sessions = proxy.list_sessions().unwrap();
        dbg!(sessions);

        assert_eq!(2 + 2, 4);
    }
}
