use std::collections::HashMap;
use std::net::{IpAddr, SocketAddrV4, Ipv4Addr};
use std::sync::Arc;
use std::time::Duration;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use thiserror::Error;
use tokio::sync::{oneshot, mpsc};
use tokio::net::UdpSocket;
use tokio::time::sleep;

const MULTICAST_PORT: u16 = 2727;
const LISTEN_ADDR: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), MULTICAST_PORT);
const MULTICAST_ADDR: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(239, 27, 27, 27), MULTICAST_PORT);
const MESSAGE: &[u8; 9] = b"BABABOOEY";

#[derive(Debug, Error)]
pub enum PeerError {
    #[error("Actor has been killed.")]
    DeadActor,
    #[error("Failed to set up listener")]
    Setup(#[from] std::io::Error)
}

pub fn make_mcast(addr: &SocketAddrV4, multi: &SocketAddrV4) -> Result<std::net::UdpSocket, PeerError> {
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;

    socket.set_reuse_address(true)?; // Allow other instances on node to use same IP.
    socket.set_reuse_port(true)?; // This doesnt seem to help, either.
    socket.bind(&SockAddr::from(*addr))?;
    //socket.set_multicast_loop_v4(true)?; // Allow discovery of self, deep bro.
    socket.join_multicast_v4(multi.ip(), addr.ip())?;
    socket.set_nonblocking(true)?; // THIS IS VERY IMPORTANT.

    Ok(socket.into())
}

async fn start_actor(mut actor: PeerManager) {
    while let Some(msg) = actor.receiver.recv().await {
        actor.handle(msg);
    }
}

// Kicks off a listener for new peers, and a broadcaster.
fn mulicaster(manager: PeerManagerHandle) -> Result<(), PeerError> {
    let std_socket = make_mcast(&LISTEN_ADDR, &MULTICAST_ADDR)?;
    let socket = UdpSocket::from_std(std_socket)?;
    let socket = Arc::new(socket);

    let listen_sock = socket.clone();
    // TODO: Is all this spawning an anti-pattern?
    tokio::spawn(async move {
        let mut buf = [0u8; 1024];
        loop {
            match listen_sock.recv_from(&mut buf).await {
                Ok((len, addr)) => {
                    if &buf[..len] == MESSAGE {
                        manager.add_peer(addr.ip()).await;
                    }
                },
                Err(e) => eprintln!("Failed to recv, {}", e)
            }
        }
    });

    let broadcast_sock = socket.clone();
    tokio::spawn(async move {
        loop {
            if let Err(e) = broadcast_sock.send_to(MESSAGE, MULTICAST_ADDR).await {
                eprintln!("Failed to send, {}", e);
            }
            sleep(Duration::from_secs(5)).await;
        }
    });


    Ok(())
}

#[derive(Default)]
struct PeerState {
    _last_seen: usize,
}

struct PeerManager {
    receiver: mpsc::Receiver<PeerMessage>,
    peers: HashMap<IpAddr, PeerState>
}

enum PeerMessage {
    NewPeer(IpAddr),
    GetPeers(oneshot::Sender<Vec<IpAddr>>)
}

impl PeerManager {
    fn new(receiver: mpsc::Receiver<PeerMessage>) -> Self {
        let peers = HashMap::new();
        Self { receiver, peers }
    }

    fn handle(&mut self, msg: PeerMessage) {
        match msg {
            PeerMessage::GetPeers(tx) => {
                let list = self.peers.keys().cloned().collect();
                tx.send(list).expect("Huh?");
            },
            PeerMessage::NewPeer(ip) => {
                match self.peers.get(&ip) {
                    Some(_state) => {},
                    None => {
                        println!("Found a new friend, {}", &ip);
                        self.peers.insert(ip, PeerState::default());
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct PeerManagerHandle {
    sender: mpsc::Sender<PeerMessage>
}

impl PeerManagerHandle {
    pub async fn new() -> Result<Self, PeerError> {
        let (tx, rx) = mpsc::channel(8);
        let actor = PeerManager::new(rx);
        let res = Self { sender: tx };

        tokio::spawn(start_actor(actor));
        mulicaster(res.clone())?;
        Ok(res)
    }

    pub async fn get_peers(&self) -> Vec<IpAddr> {
        let (tx, rx) = oneshot::channel();
        let msg = PeerMessage::GetPeers(tx);
        let _ = self.sender.send(msg).await;
        rx.await.expect("actor is kill")
    }

    async fn add_peer(&self, ip: IpAddr) {
        let msg = PeerMessage::NewPeer(ip);
        self.sender.send(msg).await.expect("actor is kill")
    }
}
