#![allow(unused)]

mod peers;

pub use peers::*;

use std::net::{SocketAddr, SocketAddrV4};
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
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
