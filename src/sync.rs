use std::error::Error;
use tokio::sync::mpsc;
use raft::{default_logger, prelude::*};
use crate::{storage::StoreHandle, PeerManagerHandle};

impl Storage for StoreHandle {
    fn initial_state(&self) -> Result<RaftState, raft::Error> {
        Ok(RaftState::new(HardState::default(), ConfState::default()))
    }

    fn entries(
        &self,
        low: u64,
        high: u64,
        max_size: impl Into<Option<u64>>,
        context: raft::GetEntriesContext,
    ) -> raft::Result<Vec<Entry>> {
        Ok(vec![])
    }

    fn term(&self, idx: u64) -> raft::Result<u64> {
        Ok(0)
    }

    fn first_index(&self) -> raft::Result<u64> {
        Ok(1)
    }

    fn last_index(&self) -> raft::Result<u64> {
        Ok(0)
    }

    fn snapshot(&self, request_index: u64, to: u64) -> raft::Result<Snapshot> {
        todo!()
    }
}

enum SyncMessage {

}

struct SyncActor {
    peers: PeerManagerHandle,
    raft: RawNode<StoreHandle>,
    receiver: mpsc::Receiver<SyncMessage>
}

impl SyncActor {

    fn handle(&mut self, msg: SyncMessage) {
        match msg {
            _ => unimplemented!()
        }
    }

    fn heartbeat(&mut self) {
        self.raft.tick();
    }
}

#[derive(Clone)]
pub struct SyncHandle {
    sender: mpsc::Sender<SyncMessage>
}

async fn run(mut actor: SyncActor) {
    let mut timer = tokio::time::interval(std::time::Duration::from_secs(10));

    loop {
        tokio::select! {
            msg = actor.receiver.recv() => match msg {
                Some(msg) => actor.handle(msg),
                None => {}
            },
            _ = timer.tick() => {
                actor.heartbeat()
            }
        }
    }
}

impl SyncHandle {
    pub async fn new(peers: PeerManagerHandle, store: StoreHandle) -> Result<Self, Box<dyn Error>> {
        let config = Config {
            id: 1, election_tick: 10, heartbeat_tick: 3, ..Default::default()
        };
        config.validate()?;
        let logger = default_logger();
        let raft = RawNode::new(&config, store.clone(), &logger)?;

        let (tx, rx) = mpsc::channel(8);
        let actor = SyncActor {
            peers, raft, receiver: rx
        };
        let res = Self { sender: tx };
        tokio::spawn(run(actor));

        Ok(res)
    }
}
