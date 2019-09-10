use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

use futures_01::{
    Future, Poll, Async, Stream,
    sync::mpsc::{Receiver, Sender},
    sink::Sink,
};

use crypto::HashValue;
use logger::prelude::*;
use failure::prelude::*;
use crypto::hash::CryptoHash;
use network::NetworkMessage;
use star_types::channel_transaction::ChannelTransaction;
use tokio::{runtime::TaskExecutor};
use std::time::{Duration, Instant};
use tokio::timer::Delay;

pub struct MessageFuture {
    rx: Receiver<ChannelTransaction>,
}

impl MessageFuture {
    pub fn new(rx: Receiver<ChannelTransaction>) -> Self {
        Self {
            rx,
        }
    }
}

impl Future for MessageFuture {
    type Item = ChannelTransaction;
    type Error = std::io::Error;

    fn poll(&mut self) -> Poll<ChannelTransaction, Self::Error> {
        while let Async::Ready(v) = self.rx.poll().unwrap() {
            match v {
                Some(v) => {
                    warn!("tx is {:?}", v);
                    return Ok(Async::Ready(v));
                }
                None => {
                    debug!("no data");
                    return Ok(Async::NotReady);
                }
            }
        };
        return Ok(Async::NotReady);
    }
}

pub struct MessageProcessor {
    tx_map: HashMap<HashValue, Sender<ChannelTransaction>>,
    executor: TaskExecutor,
}

impl MessageProcessor {
    pub fn new(executor: TaskExecutor,) -> Self {
        Self {
            tx_map: HashMap::new(),
            executor,
        }
    }

    pub fn add_future(&mut self, hash: HashValue, mut sender: Sender<ChannelTransaction>,timeout:u64) {
        self.tx_map.entry(hash).or_insert(sender.clone());

        if(timeout==0){
            return
        }
        let task = Delay::new(Instant::now() + Duration::from_millis(timeout))
            .and_then(move |_| {
                sender.close();
                Ok(())
            })
            .map_err(|e| panic!("delay errored; err={:?}", e));
        self.executor.spawn(task);
    }

    pub fn send_response(&mut self, mut msg: ChannelTransaction) -> Result<()> {
        let hash = msg.txn.clone().into_raw_transaction().hash();

        match self.tx_map.get(&hash) {
            Some(tx) => {
                match tx.clone().send(msg).wait() {
                    Ok(_new_tx) => {
                        info!("send message succ");
                        self.tx_map.remove(&hash);
                    },
                    Err(_) => warn!("send message error"),
                };
            }
            _ => info!("tx hash {} not in map", hash),
        }
        Ok(())
    }
}
