// KDE networking — split from a single 381-line file.
// trait_impl.rs calls networking::function_name directly.

mod bluetooth;
mod network;

pub(super) use bluetooth::*;
pub(super) use network::*;
