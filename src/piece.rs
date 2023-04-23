use std::{
    collections::BTreeMap,
    net::SocketAddr,
    sync::mpsc::{channel, Sender},
    thread,
};

use sha1::{Digest, Sha1};

use crate::{message, peer::Peers, BLOCK_SIZE};

#[derive(Debug)]
enum AddError {
    BlockAlignError,
    BeginNotInRage,
    BlockOversize,
    BlockOverwrite,
}
pub struct Piece {
    pub hash: [u8; 20],
    pub len: u32,
    pub complete: bool,
    pub blocks: BTreeMap<u32, Vec<u8>>,
    block_count: u32,
}

pub struct BlockParam {
    begin: u32,
    len: u32
}

impl Piece {
    pub fn new(hash: [u8; 20], len: u32) -> Piece {
        Piece {
            hash,
            len,
            complete: false,
            blocks: BTreeMap::new(),
            block_count: len / BLOCK_SIZE,
        }
    }

    pub fn unfinished_blocks(&self) -> Vec<BlockParam> {
        let finished_blocks = self.blocks.keys().collect::<Vec<_>>();
        let mut all_blocks = (0..self.block_count).collect::<Vec<_>>();
        all_blocks.retain(|i| finished_blocks.contains(&i) == false);
        all_blocks.into_iter().map(|begin|{
            let mut block_size = BLOCK_SIZE;
            if begin == self.block_count - 1 {
                block_size = self.len % BLOCK_SIZE;
            }
            BlockParam {
                begin: begin * BLOCK_SIZE,
                len: block_size
            }
        }).collect()
    }

    fn update_complete(&mut self) {
        if self.blocks.len() == self.block_count as usize {
            let mut hasher = Sha1::new();
            let blocks = self
                .blocks
                .clone()
                .into_values()
                .flatten()
                .collect::<Vec<_>>();
            hasher.update(blocks);
            let result = hasher.finalize().to_vec();
            if self.hash == result.as_slice() {
                self.complete = true;
            }
        }
    }

    pub fn add(&mut self, begin: u32, block: Vec<u8>) -> Result<(), AddError> {
        let block_index = begin / BLOCK_SIZE;
        let check_align = begin % BLOCK_SIZE;

        if check_align > 0 {
            return Err(AddError::BlockAlignError);
        }
        if block_index >= self.block_count {
            return Err(AddError::BeginNotInRage);
        }
        if block.len() > BLOCK_SIZE as usize {
            return Err(AddError::BlockOversize);
        }

        let insert_res = self.blocks.insert(block_index, block);
        self.update_complete();
        if insert_res.is_some() {
            return Err(AddError::BlockOverwrite);
        }
        Ok(())
    }
}

pub fn start_piece_receiver(
    peers: Peers,
    mut pieces: Vec<Piece>,
) -> Sender<(SocketAddr, message::Piece)> {
    let (tx, rx) = channel::<(SocketAddr, message::Piece)>();

    thread::spawn(move || {
        while let Ok((peer_addr, block)) = rx.recv() {
            if let Some(piece) = pieces.get_mut(block.index as usize) {
                if let Err(e) = piece.add(block.begin, block.block) {
                    println!("{:?}", e);
                }
            }
        }
    });
    tx
}
