use crossbeam_channel::{Receiver, Sender};
use parking_lot::{Condvar, Mutex};
use std::{
    collections::HashMap,
    fmt::Debug,
    net::{SocketAddr, TcpStream},
    sync::Arc,
    thread::{self},
    time::{Duration},
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
    //#[error("Get peer from peers hashmap error")]
    //GetPeersHashMap,
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


impl Peer {
    fn process(
        peers: Peers,
        complete_pieces: Arc<Mutex<Vec<Piece>>>,
        addr: SocketAddr,
        info_hash: Vec<u8>,
        my_id: Vec<u8>,
        get_piece: Receiver<Piece>,
        return_piece: Sender<Piece>,
    ) -> Result<(), Err> {
        let s = TcpStream::connect(addr)?;
        let p = Arc::new(PeerProto::handshake(s, &info_hash, &my_id)?);

        let msg = p.recv()?;
        let bitfield = match msg {
            Message::Bitfield(bf) => bf,
            _ => return Err(Err::BitfieldNotRecv),
        };

        peers.lock().insert(addr, Peer {});

        p.send(Message::Interested)?;

        let choke_lock = Arc::new((Mutex::new(State::Choke), Condvar::new()));

        let (msg_piece_tx, msg_piece_rx) = crossbeam_channel::unbounded();
        let pp = p.clone();
        let choke_lock2 = choke_lock.clone();
        thread::spawn(move || {
            while let Ok(msg) = pp.recv() {
                //println!("{:?} [{:?}] {:?}", Instant::now(), addr, msg);
                match msg {
                    Message::Choke => *choke_lock2.0.lock() = State::Choke,
                    Message::Unchoke => {
                        let (lock, cvar) = &*choke_lock2;
                        let mut choke = lock.lock();
                        *choke = State::Unchoke;
                        cvar.notify_one();
                    }
                    //Message::Have(h) => cfg.lock().bitfield.set(h.piece_index as usize, true),
                    //Message::Bitfield(bf) => cfg.lock().bitfield = bf,
                    Message::Piece(p) => {
                        if msg_piece_tx.send(p).is_err() {
                            break;
                        }
                    } //chan_tx.send((addr, p))?,
                    _ => (),
                }
            }
            /*let msg = p.recv().or_else(|e| {
                peers.lock().remove(&addr);
                Err(e)
            })?;*/
        });

        // waiting while choked
        let (lock, cvar) = &*choke_lock;
        let mut choke = lock.lock();
        if *choke == State::Choke {
            cvar.wait(&mut choke);
        }

        /*while let Ok(msg_piece) = msg_piece_rx.recv() {

        }*/

        while let Ok(mut piece) = get_piece.recv() {
            if bitfield.get(piece.index) != Some(true) {
                #[allow(unused_must_use)] {
                    return_piece.send(piece);
                }
                continue;
            }
            for u in piece.unfinished_blocks().chunks(8) {
                for uc in u {
                    #[allow(unused_must_use)] {
                        p.send(Message::Request(message::Request::new(
                            piece.index as u32,
                    uc.begin,
                    uc.len,
                )));
                    }
                }
                for _ in 0..8 {
                    match msg_piece_rx.recv_timeout(Duration::from_secs(10)) {
                        Ok(msg_piece) => {
                            #[allow(unused_must_use)] {
                                piece.add(msg_piece.begin, msg_piece.block);
                            }
                        }
                        Err(_) => {
                            peers.lock().remove(&addr);
                            #[allow(unused_must_use)] {
                                return_piece.send(piece);
                            }
                            return Ok(());
                        },
                    }
                }
            }
            if piece.complete {
                complete_pieces.lock().push(piece);
            } else {
                #[allow(unused_must_use)] {
                    return_piece.send(piece);
                }
            }
        }
        Ok(())
    }

    pub fn start_receiver(
        addr: SocketAddr,
        info_hash: Vec<u8>,
        my_id: Vec<u8>,
        peers: Peers,
        complete_pieces: Arc<Mutex<Vec<Piece>>>,
        peer: lava_torrent::tracker::Peer, //chan_tx: Sender<(SocketAddr, message::Piece)>,
        get_piece: Receiver<Piece>,
        return_piece: Sender<Piece>,
    ) {
        if !peers.lock().contains_key(&addr) {
            //thread::spawn(move || Peer::test(peers, addr, info_hash, peer_id));
            thread::spawn(move || {
                Peer::process(peers, complete_pieces, peer.addr, info_hash, my_id, get_piece, return_piece)
            });
        }
    }
}
