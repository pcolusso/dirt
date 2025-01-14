#![allow(unused)]

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

pub fn make_mcast(addr: &SocketAddrV4, multi: &SocketAddrV4) -> Result<std::net::UdpSocket, DirtError> {
    if !multi.ip().is_multicast() {
        return Err(DirtError::BadMulticastAddress);
    }
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;

    socket.set_reuse_address(true)?;
    socket.bind(&SockAddr::from(*addr));
    socket.set_multicast_loop_v4(true)?;
    socket.join_multicast_v4(multi.ip(), addr.ip())?;

    Ok(socket.into())
}
