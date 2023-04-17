use std::{
    net::SocketAddr,
    sync::mpsc::{channel, Sender},
    thread,
};

use crate::{message, peer::Peers};

pub struct Piece {
    pub hash: [u8; 20],
    pub complete: bool,
}

impl Piece {
    pub fn new(hash: [u8; 20]) -> Piece {
        Piece {
            hash,
            complete: false,
        }
    }
}

pub fn start_piece_receiver(peers: Peers) -> Sender<(SocketAddr, message::Piece)> {
    let (tx, rx) = channel();

    thread::spawn(move || {});

    tx
}
