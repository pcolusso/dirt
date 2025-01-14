use std::{collections::HashSet, net::{IpAddr, Ipv4Addr, SocketAddr}, sync::{Arc, Mutex}, time::Duration};
use dirt::DirtError;
use thiserror::Error;
use tokio::{net::UdpSocket, time::sleep};


#[derive(Error, Debug)]
enum AppError {
    #[error("App error; ")]
    Core(#[from] DirtError),
    #[error("I/O error")]
    Io(#[from] std::io::Error),
}

const MULTICAST_ADDR: Ipv4Addr = Ipv4Addr::new(239, 27, 27, 27);
const MULTICAST_PORT: u16 = 2727;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.join_multicast_v4(MULTICAST_ADDR, Ipv4Addr::UNSPECIFIED)?;

    let socket = Arc::new(socket);
    let peers = Arc::new(Mutex::new(HashSet::new()));

    let listen_sock = socket.clone();
    let listener_peers = peers.clone();
    tokio::spawn(async move {
        let mut buf = [0u8; 1024];
        loop {
            match listen_sock.recv_from(&mut buf).await {
                Ok((len, addr)) => {
                    if &buf[..len] ==  b"DISCOVER" {
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
        let dest = SocketAddr::new(IpAddr::V4(MULTICAST_ADDR), MULTICAST_PORT);
        loop {
            if let Err(e) = broadcast_sock.send_to(b"DISCOVER", dest).await {
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
