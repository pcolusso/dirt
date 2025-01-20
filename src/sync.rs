use raft::{default_logger, prelude::*};
use std::time::Duration;

use crate::storage::Store;

impl Storage for Store {
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

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let config = Config {
        id: 1,
        election_tick: 10,
        heartbeat_tick: 3,
        ..Default::default()
    };
    config.validate()?;
    let logger = default_logger();

    let mut raft = RawNode::new(&config, store.clone(), &logger)?;

    tokio::spawn(async move {
        loop {
            raft.tick();
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });

    store.put("key1".to_string(), "value1".to_string()).await;
    store.put("key2".to_string(), "value2".to_string()).await;

    // Print the stored values
    println!("key1: {:?}", store.get("key1").await);
    println!("key2: {:?}", store.get("key2").await);

    // Keep the main thread running
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
