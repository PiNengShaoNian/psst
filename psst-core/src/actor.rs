use crossbeam_channel::{bounded, unbounded, Receiver, RecvTimeoutError, Sender};
use std::{
    fmt::Display,
    thread::{self, JoinHandle},
    time::Duration,
};

pub enum Act<T: Actor> {
    Continue,
    WaitOr {
        timeout: Duration,
        timeout_msg: T::Message,
    },
    Shutdown,
}

pub trait Actor: Sized {
    type Message: Send + 'static;
    type Error: Display;

    fn handle(&mut self, msg: Self::Message) -> Result<Act<Self>, Self::Error>;

    fn process(mut self, recv: Receiver<Self::Message>) {
        let mut act = Act::Continue;
        loop {
            let msg = match act {
                Act::Continue => match recv.recv() {
                    Ok(msg) => msg,
                    Err(_) => {
                        break;
                    }
                },
                Act::WaitOr {
                    timeout,
                    timeout_msg,
                } => match recv.recv_timeout(timeout) {
                    Ok(msg) => msg,
                    Err(RecvTimeoutError::Timeout) => timeout_msg,
                    Err(RecvTimeoutError::Disconnected) => {
                        break;
                    }
                },
                Act::Shutdown => {
                    break;
                }
            };
            act = match self.handle(msg) {
                Ok(act) => act,
                Err(err) => {
                    log::error!("error: {}", err);
                    break;
                }
            }
        }
    }

    fn spawn<F>(cap: Capacity, name: &str, factory: F) -> ActorHandle<Self::Message>
    where
        F: FnOnce(Sender<Self::Message>) -> Self + Send + 'static,
    {
        let (send, recv) = cap.to_channel();
        ActorHandle {
            sender: send.clone(),
            thread: thread::Builder::new()
                .name(name.to_string())
                .spawn(move || {
                    factory(send).process(recv);
                })
                .unwrap(),
        }
    }
}

pub struct ActorHandle<M> {
    thread: JoinHandle<()>,
    sender: Sender<M>,
}

pub enum Capacity {
    Sync,
    Bounded(usize),
    Unbounded,
}

impl Capacity {
    pub fn to_channel<T>(&self) -> (Sender<T>, Receiver<T>) {
        match self {
            Capacity::Sync => bounded(0),
            Capacity::Bounded(cap) => bounded(*cap),
            Capacity::Unbounded => unbounded(),
        }
    }
}
