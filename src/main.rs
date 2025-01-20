use dirt::{storage::{Store, StoreHandle}, PeerError, PeerManagerHandle};
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
    let store = StoreHandle::new();
    dirt::sync::run().await.unwrap();


    Ok(())

}
