use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::mpsc::{channel, Sender},
    thread,
};

use crate::{message, peer::Peers, BLOCK_SIZE};

pub struct Piece {
    pub hash: [u8; 20],
    pub len: i64,
    pub complete: bool,
    pub blocks: HashMap<u32, [u8; BLOCK_SIZE]>,
}

impl Piece {
    pub fn new(hash: [u8; 20], len: i64) -> Piece {
        Piece {
            hash,
            len,
            complete: false,
            blocks: HashMap::new(),
        }
    }
}

pub fn start_piece_receiver(
    peers: Peers,
    pieces: Vec<Piece>,
) -> Sender<(SocketAddr, message::Piece)> {
    let (tx, rx) = channel();

    thread::spawn(move || {});

    tx
}
