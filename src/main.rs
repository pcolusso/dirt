use std::{collections::HashSet, net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4}, sync::{Arc, Mutex}, time::Duration};
use dirt::{make_mcast, DirtError, PeerError, PeerManagerHandle};
use thiserror::Error;
use tokio::{net::UdpSocket, time::sleep};

#[derive(Error, Debug)]
enum AppError {
    #[error("App error; ")]
    Core(#[from] DirtError),
    #[error("I/O error")]
    Io(#[from] std::io::Error),
    #[error("adsf")]
    Idlk(#[from] PeerError),
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let actor = PeerManagerHandle::new().await?;

    loop {
        sleep(Duration::from_secs(5)).await;
        let peer_list = actor.get_peers().await;
        println!("Discovered peers: {:?}", peer_list);
    }
}
