//! Reference https://freedesktop.org/wiki/Software/systemd/logind/
pub mod types;
pub mod proxy;

use proxy::{logind, session};
use zbus::{Connection, Result};

const DEFAULT_DEST: &str = "org.freedesktop.login1";

pub struct Logind<'a> {
    connection: Connection,
    logind_proxy: logind::ManagerProxy<'a>,
}

impl<'a> Logind<'a> {
    pub fn new() -> Result<Self> {
        let connection = Connection::new_system()?;
        let logind_proxy = logind::ManagerProxy::new(&connection)?;
        Ok(Self {
            connection,
            logind_proxy,
        })
    }

    pub fn logind(&self) -> &logind::ManagerProxy<'a> {
        &self.logind_proxy
    }

    pub fn session(&self,
    path: &'a str) -> session::SessionProxy<'a> {
        let session_proxy = session::SessionProxy::new_for(&self.connection, DEFAULT_DEST, path).unwrap();
        session_proxy
    }
}

#[cfg(test)]
mod tests {
    use crate::Logind;
    use crate::types::session::SessionType;

    #[test]
    fn basic_test() {
        let proxy = Logind::new().unwrap();

        let sessions = proxy.logind().list_sessions().unwrap();
        dbg!(&sessions);

        let session_proxy = proxy.session(sessions[0].path());
        //let res = session_proxy.seat().unwrap();
        let res = session_proxy.name().unwrap();
        dbg!(res);
        let res = session_proxy.class().unwrap();
        dbg!(res);
        let res = session_proxy.type_().unwrap();
        let e:SessionType = res.as_str().into();
        dbg!(e);
        dbg!(res);
        let res = session_proxy.active().unwrap();
        dbg!(res);
    
        assert_eq!(2 + 2, 4);
    }
}
