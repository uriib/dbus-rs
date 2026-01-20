use std::{
    borrow::Cow,
    env::{self, VarError},
    os::unix::net::UnixStream,
};

pub fn read_session_address() -> Result<Cow<'static, str>, VarError> {
    env::var("DBUS_SESSION_BUS_ADDRESS").map(Cow::Owned)
}

pub fn read_system_address() -> Cow<'static, str> {
    env::var("DBUS_SYSTEM_BUS_ADDRESS")
        .map(Cow::Owned)
        .unwrap_or("unix:path=/var/run/dbus/system_bus_socket".into())
}

pub fn read_starter_address() -> Result<String, VarError> {
    env::var("DBUS_SESSION_BUS_ADDRESS")
}

fn make_sockaddr_un(
    start: usize,
    s: &str,
) -> Result<libc::sockaddr_un, Box<dyn std::error::Error>> {
    let bytes = s.as_bytes();
    let mut r = libc::sockaddr_un {
        sun_family: libc::AF_UNIX as libc::sa_family_t,
        sun_path: [0; 108],
    };
    if start + bytes.len() + 1 >= r.sun_path.len() {
        Err("Address too long")?
    }
    for (i, &x) in bytes.into_iter().enumerate() {
        r.sun_path[i + start] = x as libc::c_char;
    }
    Ok(r)
}

pub fn address_to_sockaddr_un(s: &str) -> Result<libc::sockaddr_un, Box<dyn std::error::Error>> {
    if !s.starts_with("unix:") {
        Err("Address is not a unix socket")?
    };
    for pair in s["unix:".len()..].split(',') {
        let mut kv = pair.splitn(2, "=");
        if let Some(key) = kv.next() {
            if let Some(value) = kv.next() {
                if key == "path" {
                    return make_sockaddr_un(0, value);
                }
                if key == "abstract" {
                    return make_sockaddr_un(1, value);
                }
            }
        }
    }
    Err(format!("unsupported address type: {}", s))?
}

pub fn connect_blocking(addr: &str) -> Result<UnixStream, Box<dyn std::error::Error>> {
    let sockaddr = address_to_sockaddr_un(addr)?;
    crate::sys::connect_blocking(&sockaddr)
}

#[test]
fn bus_exists() {
    let addr = read_session_address().unwrap();
    println!("Bus address is: {:?}", addr);
    if addr.starts_with("unix:path=") {
        let path = std::path::Path::new(&addr["unix:path=".len()..]);
        assert!(path.exists());
    }

    let addr = read_system_address();
    if addr.starts_with("unix:path=") {
        let path = std::path::Path::new(&addr["unix:path=".len()..]);
        assert!(path.exists());
    }
}
