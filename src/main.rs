use std::{collections::HashSet, net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4}, sync::{Arc, Mutex}, time::Duration};
use dirt::{make_mcast, DirtError};
use thiserror::Error;
use tokio::{net::UdpSocket, time::sleep};


#[derive(Error, Debug)]
enum AppError {
    #[error("App error; ")]
    Core(#[from] DirtError),
    #[error("I/O error")]
    Io(#[from] std::io::Error),
}

// Multicast address space is 239/8
const MULTICAST_PORT: u16 = 2727;
const LISTEN_ADDR: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), MULTICAST_PORT);
const MULTICAST_ADDR: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(239, 27, 27, 27), MULTICAST_PORT);
const MESSAGE: &[u8; 9] = b"BABABOOEY";

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let std_socket = make_mcast(&LISTEN_ADDR, &MULTICAST_ADDR)?;
    let socket = UdpSocket::from_std(std_socket)?;

    let socket = Arc::new(socket);
    let peers = Arc::new(Mutex::new(HashSet::new()));

    let listen_sock = socket.clone();
    let listener_peers = peers.clone();
    tokio::spawn(async move {
        let mut buf = [0u8; 1024];
        loop {
            match listen_sock.recv_from(&mut buf).await {
                Ok((len, addr)) => {
                    if &buf[..len] == MESSAGE {
                        println!("Discovered peer {}", addr);
                        listener_peers.lock().unwrap().insert(addr.ip());
                    }
                },
                Err(e) => eprintln!("Failed to recv, {}", e)
            }
        }
    });

    let broadcast_sock = socket.clone();
    tokio::spawn(async move {
        loop {
            println!("send!");
            if let Err(e) = broadcast_sock.send_to(MESSAGE, MULTICAST_ADDR).await {
                eprintln!("Failed to send, {}", e);
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });

    loop {
        sleep(Duration::from_secs(5)).await;
        let peer_list = peers.lock().unwrap();
        println!("Discovered peers: {:?}", *peer_list);
    }
}
