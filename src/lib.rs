mod peers;

pub use peers::*;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DirtError {
    #[error("Placeholder")]
    Unknown,
    #[error("Multicast address is invalid.")]
    BadMulticastAddress,
    #[error("I/O error")]
    IO(#[from] std::io::Error)
}
