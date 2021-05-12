fn main() {
    windows::build!(
        Windows::Foundation::{AsyncActionCompletedHandler, IAsyncAction, IAsyncOperation},
        Windows::Foundation::Collections::*,
        Windows::Networking::*,
        Windows::Networking::Connectivity::*,
        Windows::Networking::ServiceDiscovery::Dnssd::*,
        Windows::Networking::Sockets::*
    );
}
