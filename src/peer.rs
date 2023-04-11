use std::{
    io::{Read, Write},
    net::{TcpStream, ToSocketAddrs},
    ops::Deref,
    sync::Arc,
    thread, time::Duration,
};

use thiserror::Error;

use crate::peer_proto::{self, PeerProto};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error while io")]
    Io(#[from] std::io::Error),
    #[error("Peer proto error")]
    PeerProto(#[from] peer_proto::Error),
}

pub struct Peer<A: ToSocketAddrs> {
    addr: A,
    info_hash: Vec<u8>,
    peer_id: Vec<u8>,
}

impl<A: ToSocketAddrs + Send + Clone+ 'static> Peer<A> {
    pub fn new(addr: A, info_hash: Vec<u8>, peer_id: Vec<u8>) -> Peer<A> {
        Peer {
            addr,
            info_hash,
            peer_id,
        }
    }

    pub fn connect(&self) -> Result<(), Error> {
        let addr = self.addr.clone();
        let info_hash = self.info_hash.clone();
        let peer_id = self.peer_id.clone();
        let mut s = TcpStream::connect_timeout(&addr.to_socket_addrs().unwrap().next().unwrap(), Duration::from_secs(1)).unwrap();
        println!("read timeout: {:?}", s.read_timeout());
        let pp = Arc::new(PeerProto::handshake(s, &info_hash, &peer_id).unwrap());
        let mut ppr = pp.clone();
        let t = thread::spawn(move || {
            //ppr.deref().recv();


        });
        t.join();

        /*let mut v = Arc::new(s);
        let mut w = v.clone();
        thread::spawn(move || {
            w.deref().write(b"qwe");
        });
        let mut buf = [0; 5];
        v.deref().read(&mut buf);*/
        Ok(())
    }

    pub fn send(&self) {}
}
