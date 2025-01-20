use std::collections::HashMap;
use tokio::sync::{mpsc, oneshot};

type Snapshot = HashMap<String, String>;

enum StoreMsg {
    Put { key: String, value: String },
    Get { key: String },
    Snapshot(Snapshot),
}

pub struct Store {
    store: HashMap<String, String>,
    receiver: mpsc::Receiver<StoreMsg>,
}

impl Store {
    fn new(receiver: mpsc::Receiver<StoreMsg>) -> Self {
        let store = HashMap::new();
        Self { store, receiver }
    }

    fn handle(&mut self, msg: StoreMsg) {
        match msg {
            _ => unimplemented!(),
        }
    }
}

#[derive(Clone)]
pub struct StoreHandle {
    sender: mpsc::Sender<StoreMsg>,
}

impl StoreHandle {
    pub async fn new() -> Self {
        let (tx, rx) = mpsc::channel(8);
        let actor = Store::new(rx);
        let res = Self { sender: tx };
        tokio::spawn(run(actor));
        res
    }
}

async fn run(mut actor: Store) {
    while let Some(msg) = actor.receiver.recv().await {
        actor.handle(msg);
    }
}
