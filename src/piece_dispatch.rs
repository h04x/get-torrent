use std::sync::Arc;

use crossbeam_channel::{Receiver, Sender};
use lava_torrent::torrent::v1::Torrent;
use parking_lot::Mutex;

use crate::piece;


pub type CompletePiece = Arc<Mutex<Vec<piece::Piece>>>;
pub struct PieceDispatch {
    pub tx: Sender<piece::Piece>,
    pub rx: Receiver<piece::Piece>,
    pub complete_piece: CompletePiece
}

impl PieceDispatch {
    pub fn new(torrent: &Torrent) -> PieceDispatch {
        let (tx, rx) = crossbeam_channel::unbounded();
        for (index, hash) in torrent.pieces.iter().enumerate() {
            let mut len = torrent.piece_length as u32;
            // last piece may be shorter than others
            if index as i64 * torrent.piece_length + torrent.piece_length > torrent.length {
                len = (torrent.length % torrent.piece_length) as u32;
            }
            tx.send(piece::Piece::new(
                index,
                hash.clone().try_into().expect("piece hash mismatch length"),
                len,
            ))
            .expect("Piece queue send exception");
        }
        let complete_piece = Arc::new(Mutex::new(Vec::new()));
        PieceDispatch { tx, rx, complete_piece }
    }
}
