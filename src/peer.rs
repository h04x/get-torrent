use crossbeam_channel::{Receiver, Sender};
use parking_lot::Mutex;
use std::{
    collections::HashMap,
    fmt::Debug,
    net::{SocketAddr, TcpStream},
    sync::Arc,
    thread::{self},
    time::{Duration, Instant},
};

use thiserror::Error;

use crate::{
    message,
    peer_proto::{self, Message, PeerProto},
    piece::Piece,
};

/*#[derive(Debug)]
pub struct Error {
    addr: std::net::SocketAddr,
    err: Err,
}*/

#[derive(Error, Debug)]
pub enum Err {
    #[error("Error while io")]
    Io(#[from] std::io::Error),
    #[error("Peer proto error")]
    PeerProto(#[from] peer_proto::Error),
    #[error("Receive message error")]
    RecvMsg(#[from] peer_proto::RecvMsgError),
    #[error("Get peer from peers hashmap error")]
    GetPeersHashMap,
    //#[error("Error while piece channel sending")]
    //PieceChannelSend(#[from] SendError<(SocketAddr, message::Piece)>),
    #[error("Bitfield not received")]
    BitfieldNotRecv,
}

#[derive(PartialEq)]
pub enum State {
    Choke,
    Unchoke,
}
pub struct Peer {}

pub type Peers = Arc<Mutex<HashMap<SocketAddr, Peer>>>;

macro_rules! peer_get_mut {
    ( $peers:expr, $addr:expr ) => {
        $peers.lock().get_mut($addr).ok_or(Err::GetPeersHashMap)?
    };
}

impl Peer {
    /*pub fn test(
        peers: Arc<Mutex<HashMap<SocketAddr, Peer>>>,
        addr: SocketAddr,
        info_hash: Vec<u8>,
        peer_id: Vec<u8>,
        //chan_tx: Sender<(SocketAddr, message::Piece)>,
    ) -> Result<(), Err> {
        let s = TcpStream::connect(addr)?;
        let p = Arc::new(PeerProto::handshake(s, &info_hash, &peer_id)?);

        let msg = p.recv()?;
        let bf = match msg {
            Message::Bitfield(bf) => bf,
            _ => return Err(Err::BitfieldNotRecv),
        };

        let pp = p.clone();
        {
            peers.lock().insert(
                addr,
                Peer {
                    proto: pp,
                    bitfield: bf,
                    choke: State::Choke,
                },
            );
        }

        p.send(Message::Interested)?;

        let pp = p.clone();
        thread::spawn(move || -> Result<(), Err> {
            loop {
                thread::sleep(Duration::from_secs(7));
                pp.send(Message::KeepAlive)?;
            }
        });

        loop {
            let msg = p.recv().or_else(|e| {
                peers.lock().remove(&addr);
                Err(e)
            })?;
            //println!("{:?} [{:?}] {:?}", Instant::now(), addr, msg);
            match msg {
                Message::Choke => peer_get_mut!(peers, &addr).choke = State::Choke,
                Message::Unchoke => peer_get_mut!(peers, &addr).choke = State::Unchoke,
                Message::Have(h) => peer_get_mut!(peers, &addr)
                    .bitfield
                    .set(h.piece_index as usize, true),
                Message::Bitfield(bf) => peer_get_mut!(peers, &addr).bitfield = bf,
                Message::Piece(p) => (), //chan_tx.send((addr, p))?,
                _ => (),
            }
        }
    }*/

    fn process(
        peers: Peers,
        addr: SocketAddr,
        info_hash: Vec<u8>,
        my_id: Vec<u8>,
        get_piece: Receiver<Piece>,
        return_piece: Sender<Piece>,
    ) -> Result<(), Err> {
        let s = TcpStream::connect(addr)?;
        let p = PeerProto::handshake(s, &info_hash, &my_id)?;

        let msg = p.recv()?;
        let mut bitfield = match msg {
            Message::Bitfield(bf) => bf,
            _ => return Err(Err::BitfieldNotRecv),
        };

        struct Cfg {
            choke: State,
            bitfield: message::Bitfield
        }

        let cfg = Arc::new(Mutex::new(Cfg{ bitfield, choke: State::Choke }));

        peers.lock().insert(addr, Peer {});

        p.send(Message::Interested)?;

        while let Ok(piece) = get_piece.recv() {

        }

        thread::spawn(move || {});

        loop {
            if cfg.lock().choke == State::Unchoke /*|| wait_unchoke.*/ {

            }

        }

        loop {
            let msg = p.recv().or_else(|e| {
                peers.lock().remove(&addr);
                Err(e)
            })?;
            println!("{:?} [{:?}] {:?}", Instant::now(), addr, msg);
            match msg {
                Message::Choke => cfg.lock().choke = State::Choke,
                Message::Unchoke => cfg.lock().choke = State::Unchoke,
                Message::Have(h) => cfg.lock().bitfield.set(h.piece_index as usize, true),
                Message::Bitfield(bf) => cfg.lock().bitfield = bf,
                Message::Piece(p) => (), //chan_tx.send((addr, p))?,
                _ => (),
            }
        }
    }

    pub fn start_receiver(
        addr: SocketAddr,
        info_hash: Vec<u8>,
        my_id: Vec<u8>,
        peers: Peers,
        peer: lava_torrent::tracker::Peer, //chan_tx: Sender<(SocketAddr, message::Piece)>,
        get_piece: Receiver<Piece>,
        return_piece: Sender<Piece>,
    ) {
        if peers.lock().contains_key(&addr) == false {
            //thread::spawn(move || Peer::test(peers, addr, info_hash, peer_id));
            thread::spawn(move || {
                Peer::process(peers, peer.addr, info_hash, my_id, get_piece, return_piece)
            });
        }
    }
}
