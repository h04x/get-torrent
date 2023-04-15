use std::{
    collections::HashMap,
    fmt::Debug,
    io::{Read, Write},
    net::{SocketAddr, TcpStream, ToSocketAddrs},
    ops::Deref,
    sync::{Arc, Mutex, MutexGuard},
    thread::{self, JoinHandle},
    time::Duration,
};

use thiserror::Error;

use crate::{
    message,
    peer_proto::{self, Message, PeerProto},
};

#[derive(Debug)]
pub struct Error {
    addr: std::net::SocketAddr,
    err: Err,
}

#[derive(Error, Debug)]
pub enum Err {
    #[error("Error while io")]
    Io(#[from] std::io::Error),
    #[error("Peer proto error")]
    PeerProto(#[from] peer_proto::Error),
    #[error("Receive message error")]
    RecvMsg(#[from] peer_proto::RecvMsgError),
    #[error("Mutex locking error")]
    LockPeersMutex,
    #[error("Get peer from peers hashmap error")]
    GetPeersHashMap,
}

pub struct Peer {
    pub proto: Arc<PeerProto>,
    bitfield: Option<message::Bitfield>,
}

type Peers = Arc<Mutex<HashMap<SocketAddr, Peer>>>;

impl Peer {
    pub fn start_receiver(
        addr: SocketAddr,
        info_hash: Vec<u8>,
        peer_id: Vec<u8>,
        peers: Peers,
    ) -> JoinHandle<Result<(), Error>> {
        let t = thread::spawn(move || {
            let s = TcpStream::connect(addr).map_err(|e| Error {
                addr,
                err: e.into(),
            })?;
            let p = Arc::new(
                PeerProto::handshake(s, &info_hash, &peer_id).map_err(|e| Error {
                    addr,
                    err: e.into(),
                })?,
            );

            let pp = p.clone();
            {
                let mut lock = peers.lock().map_err(|e| Error {
                    addr,
                    err: Err::LockPeersMutex,
                })?;
                lock.insert(
                    addr,
                    Peer {
                        proto: pp,
                        bitfield: None,
                    },
                );
            }

            let ppp = p.clone();

            thread::spawn(move || loop {
                thread::sleep(Duration::from_secs(5));
                ppp.send(Message::KeepAlive).unwrap();
            });

            p.send(Message::Interested).map_err(|e| Error {
                addr,
                err: e.into(),
            })?;

            loop {
                let msg = p.recv().map_err(|e| Error {
                    addr,
                    err: e.into(),
                })?;
                println!("[{:?}] {:?}", addr, msg);
                match msg {
                    Message::Choke => (),
                    Message::Unchoke => (),
                    Message::Have(h) => (),
                    Message::Bitfield(bf) => {
                        peers
                            .lock()
                            .map_err(|e| Error {
                                addr,
                                err: Err::LockPeersMutex,
                            })?
                            .get_mut(&addr)
                            .ok_or(Error {
                                addr,
                                err: Err::GetPeersHashMap,
                            })?
                            .bitfield = Some(bf);
                    }
                    Message::Piece(p) => (),
                    _ => (),
                }
            }
            //Ok(())
        });
        /*let addr = self.addr.clone();
        let info_hash = self.info_hash.clone();
        let peer_id = self.peer_id.clone();
        let mut s = TcpStream::connect_timeout(&addr.to_socket_addrs().unwrap().next().unwrap(), Duration::from_secs(1)).unwrap();
        println!("read timeout: {:?}", s.read_timeout());
        let pp = Arc::new(PeerProto::handshake(s, &info_hash, &peer_id).unwrap());
        let mut ppr = pp.clone();
        let t = thread::spawn(move || {
            //ppr.deref().recv();


        });
        t.join();*/

        /*let mut v = Arc::new(s);
        let mut w = v.clone();
        thread::spawn(move || {
            w.deref().write(b"qwe");
        });
        let mut buf = [0; 5];
        v.deref().read(&mut buf);*/
        t
    }

    pub fn send(&self) {}
}
