use std::collections::HashMap;
use std::net::{IpAddr, SocketAddrV4, Ipv4Addr};
use std::sync::Arc;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use thiserror::Error;
use tokio::sync::{oneshot, mpsc};
use tokio::net::UdpSocket;
use tokio::time::sleep;

#[derive(Serialize, Deserialize)]
struct Payload {
    header: Header,
    message: Message
}

impl Payload {
    fn discover() -> Self {
        Self {
            header: Header { version: 0 },
            message: Message::Discover
        }
    }

    fn hello() -> Self {
        Self {
            header: Header { version: 0 },
            message: Message::Greet
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Header {
    version: u8
}

#[derive(Serialize, Deserialize)]
enum Message {
    Discover,
    Greet,
    Check,
}

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
    socket.set_multicast_loop_v4(true)?; // Allow discovery of self, deep bro.
    socket.join_multicast_v4(multi.ip(), addr.ip())?;
    socket.set_nonblocking(true)?; // THIS IS VERY IMPORTANT.

    Ok(socket.into())
}

async fn start_actor(mut actor: Manager) {
    while let Some(msg) = actor.receiver.recv().await {
        actor.handle(msg);
    }
}

async fn handle_incoming(sock: &UdpSocket, manager: &PeerManagerHandle) -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = [0u8; 1024];
    let (len, addr) = sock.recv_from(&mut buf).await?;
    let payload: Payload = bincode::deserialize(&buf[..len])?;

    match payload.message {
        Message::Discover => {
            manager.add_peer(addr.ip()).await;
            let encoded = bincode::serialize(&Payload::hello()).unwrap();
            sock.send_to(&encoded, addr).await?;
        },
        Message::Greet => {
            manager.add_peer(addr.ip()).await;
        },
        Message::Check => {}
    }

    Ok(())
}

// Kicks off a listener for new peers, and a broadcaster.
fn mulicaster(manager: PeerManagerHandle) -> Result<(), PeerError> {
    let std_socket = make_mcast(&LISTEN_ADDR, &MULTICAST_ADDR)?;
    let socket = UdpSocket::from_std(std_socket)?;
    let socket = Arc::new(socket);

    let listen_sock = socket.clone();
    // TODO: Is all this spawning an anti-pattern?
    tokio::spawn(async move {
        loop {
            if let Err(e) = handle_incoming(&listen_sock, &manager).await {
                eprintln!("{}", e)
            }
        }
    });

    let broadcast_sock = socket.clone();
    tokio::spawn(async move {
        let encoded = bincode::serialize(&Payload::discover()).unwrap();
        loop {

            if let Err(e) = broadcast_sock.send_to(&encoded, MULTICAST_ADDR).await {
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

struct Manager {
    receiver: mpsc::Receiver<ManagerMsg>,
    peers: HashMap<IpAddr, PeerState>
}

enum ManagerMsg {
    NewPeer(IpAddr),
    GetPeers(oneshot::Sender<Vec<IpAddr>>)
}

impl Manager {
    fn new(receiver: mpsc::Receiver<ManagerMsg>) -> Self {
        let peers = HashMap::new();
        Self { receiver, peers }
    }

    fn handle(&mut self, msg: ManagerMsg) {
        match msg {
            ManagerMsg::GetPeers(tx) => {
                let list = self.peers.keys().cloned().collect();
                tx.send(list).expect("Huh?");
            },
            ManagerMsg::NewPeer(ip) => {
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
    sender: mpsc::Sender<ManagerMsg>
}

impl PeerManagerHandle {
    pub async fn new() -> Result<Self, PeerError> {
        let (tx, rx) = mpsc::channel(8);
        let actor = Manager::new(rx);
        let res = Self { sender: tx };

        tokio::spawn(start_actor(actor));
        mulicaster(res.clone())?;
        Ok(res)
    }

    pub async fn get_peers(&self) -> Vec<IpAddr> {
        let (tx, rx) = oneshot::channel();
        let msg = ManagerMsg::GetPeers(tx);
        let _ = self.sender.send(msg).await;
        rx.await.expect("actor is kill")
    }

    async fn add_peer(&self, ip: IpAddr) {
        let msg = ManagerMsg::NewPeer(ip);
        self.sender.send(msg).await.expect("actor is kill")
    }
}
