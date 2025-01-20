use std::time::Duration;

use dirt::{storage::{Store, StoreHandle}, sync::SyncHandle, PeerError, PeerManagerHandle};
use thiserror::Error;

#[derive(Error, Debug)]
enum AppError {
    #[error("I/O error")]
    Io(#[from] std::io::Error),
    #[error("adsf")]
    Idlk(#[from] PeerError),
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<(), AppError> {
    //console_subscriber::init();

    let actor = PeerManagerHandle::new().await?;
    let store = StoreHandle::new().await;
    let sync = SyncHandle::new(actor, store).await;

    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

}
