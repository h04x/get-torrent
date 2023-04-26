use std::{
    collections::BTreeMap,
};
use sha1::{Digest, Sha1};

use crate::{
    BLOCK_SIZE,
};

#[derive(Debug)]
pub enum AddError {
    BlockAlignError,
    BeginNotInRage,
    BlockOversize,
    BlockOverwrite,
}

#[derive(Debug)]
pub struct Piece {
    pub index: usize,
    pub hash: [u8; 20],
    pub len: u32,
    pub complete: bool,
    pub blocks: BTreeMap<u32, Vec<u8>>,
    block_count: u32,
}

#[derive(Debug)]
pub struct BlockParam {
    pub begin: u32,
    pub len: u32,
}

impl Piece {
    pub fn new(index: usize, hash: [u8; 20], len: u32) -> Piece {
        Piece {
            index,
            hash,
            len,
            complete: false,
            blocks: BTreeMap::new(),
            block_count: (len + BLOCK_SIZE - 1) / BLOCK_SIZE, //divceil
        }
    }

    pub fn unfinished_blocks(&self) -> Vec<BlockParam> {
        let finished_blocks = self.blocks.keys().collect::<Vec<_>>();
        let mut all_blocks = (0..self.block_count).collect::<Vec<_>>();
        all_blocks.retain(|i| !finished_blocks.contains(&i));
        all_blocks
            .into_iter()
            .map(|begin| {
                let mut block_size = BLOCK_SIZE;
                // last block may be shorter than others
                if (begin * BLOCK_SIZE) + BLOCK_SIZE > self.len {
                    block_size = self.len - begin * BLOCK_SIZE;
                }
                BlockParam {
                    begin: begin * BLOCK_SIZE,
                    len: block_size,
                }
            })
            .collect()
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

/*pub fn start_piece_receiver(peers: Peers, pieces: Pieces) -> Sender<(SocketAddr, message::Piece)> {
    let (tx, rx) = channel::<(SocketAddr, message::Piece)>();

    thread::spawn(move || {
        let unfinished_blocks = pieces.lock().get(0).unwrap().unfinished_blocks();
        let block = unfinished_blocks.first().unwrap();
        loop {
            thread::sleep(Duration::from_secs(1));
            {
                let lock = peers.lock();
                if lock.len() > 0 {
                    lock.values()
                        .last()
                        .unwrap()
                        .proto
                        .send(Message::Request(Request::new(0, block.begin, block.len)))
                        .unwrap();
                    break;
                }
            }
        }

        while let Ok((peer_addr, block)) = rx.recv() {
            if let Some(piece) = pieces.lock().get_mut(block.index as usize) {
                if let Err(e) = piece.add(block.begin, block.block) {
                    println!("{:?}", e);
                }
            }
        }
    });
    tx
}*/
