use crossbeam_channel::{Receiver, SendError, Sender};
use lava_torrent::tracker::TrackerResponse;
use parking_lot::{Condvar, Mutex};
use std::{
    collections::HashMap,
    fmt::Debug,
    net::{IpAddr, SocketAddr, TcpStream, UdpSocket},
    sync::Arc,
    thread::{self},
    time::{Duration, Instant},
};

use thiserror::Error;

use crate::{
    //peer_proto::message,
    //peer_proto::{self, message::Extended, Message, PeerProto},
    piece::Piece,
    piece_dispatch::CompletePiece,
    PARALLEL_REQUEST_PER_PEER,
};

use bittorrent_peer_proto::{
    message::{self, Extended},
    peer_proto,
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

#[derive(Error, Debug)]
pub enum RunError {
    #[error("cannot extract peers from response")]
    TrackerResponse,
    #[error("cannot fill sockaddr channel")]
    SendError(#[from] SendError<SocketAddr>),
}

type ActivePeers = Arc<Mutex<HashMap<SocketAddr, ()>>>;
type ChokeLock = Arc<(Mutex<State>, Condvar)>;
pub struct PeerDispatch {
    pub send_peer: Sender<SocketAddr>,
    pub get_peer: Receiver<SocketAddr>,
    pub active_peers: ActivePeers,
}

impl PeerDispatch {
    pub fn run(
        resp: &TrackerResponse,
        info_hash: [u8; 20],
        local_peer_id: [u8; 20],
        get_piece: Receiver<Piece>,
        return_piece: Sender<Piece>,
        complete_piece: CompletePiece,
        msg_port_send: Sender<(SocketAddr, message::Port)>,
    ) -> Result<PeerDispatch, RunError> {
        let (send_peer, get_peer) = crossbeam_channel::unbounded();

        // TODO: start periodically poll tracker
        match resp {
            TrackerResponse::Success { peers, .. } => {
                for p in peers {
                    send_peer.send(p.addr)?;
                }
            }
            TrackerResponse::Failure { .. } => return Err(RunError::TrackerResponse),
        }

        let active_peers = Arc::new(Mutex::new(HashMap::new()));

        let ap = active_peers.clone();
        let gp = get_peer.clone();
        let sp = send_peer.clone();
        thread::spawn(move || {
            Self::peer_receiver(
                ap,
                gp,
                sp,
                info_hash,
                local_peer_id,
                get_piece,
                return_piece,
                complete_piece,
                msg_port_send,
            )
        });

        Ok(PeerDispatch {
            send_peer,
            get_peer,
            active_peers,
        })
    }

    fn peer_receiver(
        active_peers: ActivePeers,
        get_peer: Receiver<SocketAddr>,
        send_peer: Sender<SocketAddr>,
        info_hash: [u8; 20],
        local_peer_id: [u8; 20],
        get_piece: Receiver<Piece>,
        return_piece: Sender<Piece>,
        complete_piece: CompletePiece,
        msg_port_send: Sender<(SocketAddr, message::Port)>,
    ) {
        while let Ok(addr) = get_peer.recv() {
            if active_peers.lock().contains_key(&addr) {
                continue;
            }
            let ap = active_peers.clone();
            let ih = info_hash.clone();
            let pi = local_peer_id.clone();
            let gp = get_piece.clone();
            let rp = return_piece.clone();
            let cp = complete_piece.clone();
            let sp = send_peer.clone();
            let mp = msg_port_send.clone();
            thread::spawn(move || Self::peer_run(ap, addr, ih, pi, gp, rp, cp, sp, mp));
        }
    }

    fn peer_run(
        active_peers: ActivePeers,
        addr: SocketAddr,
        info_hash: [u8; 20],
        local_peer_id: [u8; 20],
        get_piece: Receiver<Piece>,
        return_piece: Sender<Piece>,
        complete_piece: CompletePiece,
        send_peer: Sender<SocketAddr>,
        msg_port_send: Sender<(SocketAddr, message::Port)>,
    ) -> Result<(), Err> {
        let s = TcpStream::connect(addr)?;
        let t = peer_proto::PeerProto::handshake(s, info_hash, local_peer_id);
        if t.is_err() {
            println!(
                "peer {} connected but handshake failed due to {:?}",
                addr, t
            );
        }
        let p = Arc::new(t?);

        let msg = p.recv()?;
        let bitfield = match msg {
            peer_proto::Message::Bitfield(bf) => bf,
            _ => return Err(Err::BitfieldNotRecv),
        };

        active_peers.lock().insert(addr, ());
        //println!("{:?}", p.peer_handshake.extended_support());

        p.send(peer_proto::Message::Interested)?;

        let choke_lock = Arc::new((Mutex::new(State::Choke), Condvar::new()));

        let (msg_piece_tx, msg_piece_rx) = crossbeam_channel::unbounded();
        let pp = p.clone();
        let cl = choke_lock.clone();
        thread::spawn(move || {
            Self::preprocess_received_msg(cl, pp, msg_piece_tx, send_peer, msg_port_send)
        });

        // waiting while choked
        let (lock, cvar) = &*choke_lock;
        let mut choke = lock.lock();
        if *choke == State::Choke {
            cvar.wait(&mut choke);
        }

        //thread::sleep(Duration::from_secs(300));

        while let Ok(mut piece) = get_piece.recv() {
            if bitfield.get(piece.index) != Some(true) {
                #[allow(unused_must_use)]
                {
                    return_piece.send(piece);
                }
                continue;
            }
            for u in piece.unfinished_blocks().chunks(PARALLEL_REQUEST_PER_PEER) {
                for uc in u {
                    #[allow(unused_must_use)]
                    {
                        p.send(peer_proto::Message::Request(message::Request::new(
                            piece.index as u32,
                            uc.begin,
                            uc.len,
                        )));
                    }
                }
                for _ in 0..PARALLEL_REQUEST_PER_PEER {
                    match msg_piece_rx.recv_timeout(Duration::from_secs(10)) {
                        Ok(msg_piece) => {
                            #[allow(unused_must_use)]
                            {
                                piece.add(msg_piece.begin, msg_piece.block);
                            }
                        }
                        Err(_) => {
                            active_peers.lock().remove(&addr);
                            #[allow(unused_must_use)]
                            {
                                return_piece.send(piece);
                            }
                            return Ok(());
                        }
                    }
                }
            }
            if piece.complete {
                complete_piece.lock().push(piece);
            } else {
                #[allow(unused_must_use)]
                {
                    return_piece.send(piece);
                }
            }
        }
        Ok(())
    }

    fn preprocess_received_msg(
        choke_lock: ChokeLock,
        peer_proto: Arc<peer_proto::PeerProto>,
        msg_piece_tx: Sender<message::Piece>,
        send_peer: Sender<SocketAddr>,
        msg_port_send: Sender<(SocketAddr, message::Port)>,
    ) {
        let s = UdpSocket::bind("0.0.0.0:0").unwrap();
        while let Ok(msg) = peer_proto.recv() {
            //println!("{:?} [{:?}] {:?}", Instant::now(), addr, msg);
            match msg {
                peer_proto::Message::Choke => *choke_lock.0.lock() = State::Choke,
                peer_proto::Message::Unchoke => {
                    let (lock, cvar) = &*choke_lock;
                    let mut choke = lock.lock();
                    *choke = State::Unchoke;
                    cvar.notify_one();
                }
                //Message::Have(h) => cfg.lock().bitfield.set(h.piece_index as usize, true),
                //Message::Bitfield(bf) => cfg.lock().bitfield = bf,
                peer_proto::Message::Piece(p) => {
                    if msg_piece_tx.send(p).is_err() {
                        break;
                    }
                } //chan_tx.send((addr, p))?,
                peer_proto::Message::Port(port) => {
                    if let Ok(addr) = peer_proto.stream.peer_addr() {
                        msg_port_send.send((addr, port));
                    }
                    /*
                    println!(
                        "port received: {:?} {:?}",
                        peer_proto.stream.peer_addr(),
                        port
                    );*/
                    if let IpAddr::V4(addr) = peer_proto.stream.peer_addr().unwrap().ip() {
                        let mut buf = Vec::new();
                        buf.extend_from_slice(&addr.octets());
                        buf.extend_from_slice(&port.bytes());
                        s.send_to(&buf, "127.0.0.1:56565");
                    }
                }
                peer_proto::Message::Extended(Extended::UtPex(pex)) => {
                    for addr in pex.added {
                        send_peer.send(addr);
                    }
                }
                peer_proto::Message::Unknown(r) => println!("Received unknown msg: {:?}", r),
                _ => (),
            }
        }
    }
}
