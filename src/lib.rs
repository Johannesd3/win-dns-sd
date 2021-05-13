mod bindings {
    windows::include_bindings!();
}

use std::convert::TryFrom;

use thiserror::Error;

use self::bindings::Windows::Networking::{
    HostName as WinHostName,
    ServiceDiscovery::Dnssd::{DnssdRegistrationStatus, DnssdServiceInstance},
    Sockets::DatagramSocket,
};

/// Windows representation of a hostname.
///
/// `TryFrom<&str>` is currently the only way to create it.
pub struct Hostname(WinHostName);

impl TryFrom<&str> for Hostname {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        WinHostName::CreateHostName(value)
            .map(Hostname)
            .map_err(ErrorInner::WindowsError)
            .map_err(Error)
    }
}

impl std::fmt::Debug for Hostname {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl std::fmt::Display for Hostname {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.CanonicalName().unwrap())
    }
}

#[derive(Debug, Error)]
enum ErrorInner {
    #[error(transparent)]
    WindowsError(#[from] windows::Error),
    #[error("Cannot register service: Invalid service name")]
    InvalidServiceName,
    #[error("Cannot register service: Server error")]
    ServerError,
    #[error("Cannot register service: Security error")]
    SecurityError,
    #[error("Cannot register service: Unknown error")]
    UnkownError,
}

/// Represents an error.
#[derive(Debug, Error)]
#[error(transparent)]
pub struct Error(ErrorInner);

type ServiceInner = (DnssdServiceInstance, DatagramSocket);

async fn register(
    instance_name: &str,
    hostname: Option<WinHostName>,
    port: u16,
    txt: &[(&str, &str)],
) -> Result<ServiceInner, ErrorInner> {
    let instance = DnssdServiceInstance::Create(instance_name, hostname, port)?;

    let txt_map = instance.TextAttributes()?;
    for &(key, value) in txt {
        txt_map.Insert(key, value)?;
    }

    let socket = DatagramSocket::new()?;
    let result = instance
        .RegisterDatagramSocketAsync1(&socket)?
        .await?
        .Status()?;

    match result {
        DnssdRegistrationStatus::Success => (),
        DnssdRegistrationStatus::InvalidServiceName => return Err(ErrorInner::InvalidServiceName),
        DnssdRegistrationStatus::SecurityError => return Err(ErrorInner::SecurityError),
        DnssdRegistrationStatus::ServerError => return Err(ErrorInner::ServerError),
        _ => return Err(ErrorInner::UnkownError),
    }

    Ok((instance, socket))
}

/// A registered DNS-SD service.
pub struct Service(ServiceInner);

impl Service {
    /// Registers a service.
    ///
    /// `instance_name` is usually something like `"instancename._http._tcp.local."`.
    pub async fn register(
        instance_name: &str,
        hostname: Option<Hostname>,
        port: u16,
        txt: &[(&str, &str)],
    ) -> Result<Self, Error> {
        register(instance_name, hostname.map(|x| x.0), port, txt)
            .await
            .map(Service)
            .map_err(Error)
    }
}
