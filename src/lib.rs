mod bindings {
    windows::include_bindings!();
}

use std::convert::TryFrom;
use std::fmt;

use bindings::Windows::Networking::{
    HostName as WinHostName, ServiceDiscovery::Dnssd::DnssdServiceInstance, Sockets::DatagramSocket,
};

async fn register(
    name: &str,
    hostname: Option<WinHostName>,
    port: u16,
    txt: &[(&str, &str)],
) -> windows::Result<(DnssdServiceInstance, DatagramSocket)> {
    let instance = DnssdServiceInstance::Create(name, hostname, port)?;

    let txt_map = instance.TextAttributes()?;
    for &(key, value) in txt {
        txt_map.Insert(key, value)?;
    }

    let socket = DatagramSocket::new()?;
    instance.RegisterDatagramSocketAsync1(&socket)?.await?;

    Ok((instance, socket))
}

pub struct Hostname(WinHostName);

impl TryFrom<&str> for Hostname {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        WinHostName::CreateHostName(value)
            .map(Hostname)
            .map_err(Error)
    }
}

#[derive(Debug)]
pub struct Error(windows::Error);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

pub struct Service((DnssdServiceInstance, DatagramSocket));

impl Service {
    pub async fn register(
        name: &str,
        hostname: Option<Hostname>,
        port: u16,
        txt: &[(&str, &str)],
    ) -> Result<Self, Error> {
        register(name, hostname.map(|x| x.0), port, txt)
            .await
            .map(Service)
            .map_err(Error)
    }
}
