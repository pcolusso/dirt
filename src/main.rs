use std::time::Duration;
use dirt::{DirtError, PeerError, PeerManagerHandle};
use thiserror::Error;
use tokio::time::sleep;

#[derive(Error, Debug)]
enum AppError {
    #[error("App error; ")]
    Core(#[from] DirtError),
    #[error("I/O error")]
    Io(#[from] std::io::Error),
    #[error("adsf")]
    Idlk(#[from] PeerError),
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<(), AppError> {
    //console_subscriber::init();

    let actor = PeerManagerHandle::new().await?;

    loop {
        let peer_list = actor.get_peers().await;
        //println!("Discovered peers: {:?}", peer_list);
        sleep(Duration::from_millis(300)).await;
    }
}
