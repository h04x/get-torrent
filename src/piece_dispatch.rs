use crossbeam_channel::{Receiver, Sender};
use lava_torrent::torrent::v1::Torrent;

use crate::piece;

pub struct PieceDispatch {
    pub tx: Sender<piece::Piece>,
    pub rx: Receiver<piece::Piece>,
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
        PieceDispatch { tx, rx }
    }
}
