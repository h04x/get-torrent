use core::fmt;

use thiserror::Error;

pub struct Bitfield {
    bitfield: Vec<u8>,
}

impl fmt::Debug for Bitfield {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Bitfield")
            .field("bitfield(bits)", &(self.bitfield.len() * 8))
            .finish()
    }
}

impl Bitfield {
    pub fn try_from_bytes(raw: Vec<u8>) -> Result<Bitfield, Error> {
        if raw.len() < 1 {
            return Err(Error::InvalidMsgLen);
        }
        Ok(Bitfield {
            bitfield: raw[1..].to_vec(),
        })
    }
    pub fn bytes(&self) -> Vec<u8> {
        unimplemented!()
    }
    pub fn bit(&self, n: usize) -> Option<bool> {
        let byte = n / 8;
        let bit = n % 8;
        Some((self.bitfield.get(byte)? >> (7 - bit)) & 1 == 1)
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Inmvalid message length")]
    InvalidMsgLen,
}

#[derive(Debug)]
pub struct Have {
    piece_index: u32,
}

impl Have {
    pub fn try_from_bytes(raw: Vec<u8>) -> Result<Have, Error> {
        if raw.len() != 5 {
            return Err(Error::InvalidMsgLen);
        }
        Ok(Have {
            piece_index: u32::from_be_bytes(raw[1..].try_into().unwrap()),
        })
    }
    pub fn bytes(&self) -> Vec<u8> {
        let mut raw = vec![4];
        raw.extend_from_slice(&self.piece_index.to_be_bytes());
        return raw;
    }
}

#[derive(Debug)]
pub struct Request {
    index: u32,
    begin: u32,
    len: u32,
}

impl Request {
    pub fn new(index: u32, begin: u32, len: u32) -> Request {
        Request { index, begin, len }
    }
    pub fn try_from_bytes(raw: Vec<u8>) -> Result<Request, Error> {
        if raw.len() != 13 {
            return Err(Error::InvalidMsgLen);
        }
        Ok(Request {
            index: u32::from_be_bytes(raw[1..5].try_into().unwrap()),
            begin: u32::from_be_bytes(raw[5..9].try_into().unwrap()),
            len: u32::from_be_bytes(raw[9..].try_into().unwrap()),
        })
    }
    pub fn bytes(&self) -> Vec<u8> {
        let mut raw = vec![6];
        raw.extend_from_slice(&self.index.to_be_bytes());
        raw.extend_from_slice(&self.begin.to_be_bytes());
        raw.extend_from_slice(&self.len.to_be_bytes());
        return raw;
    }
}

pub struct Piece {
    index: u32,
    begin: u32,
    block: Vec<u8>,
}

impl fmt::Debug for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Piece")
            .field("index", &self.index)
            .field("begin", &self.begin)
            .field("block(bytes)", &self.block.len())
            .finish()
    }
}

impl Piece {
    pub fn try_from_bytes(raw: Vec<u8>) -> Result<Piece, Error> {
        if raw.len() < 9 {
            return Err(Error::InvalidMsgLen);
        }
        Ok(Piece {
            index: u32::from_be_bytes(raw[1..5].try_into().unwrap()),
            begin: u32::from_be_bytes(raw[5..9].try_into().unwrap()),
            block: raw[9..].to_vec(),
        })
    }
    pub fn bytes(&self) -> Vec<u8> {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct Cancel {
    index: u32,
    begin: u32,
    len: u32,
}

impl Cancel {
    pub fn try_from_bytes(raw: Vec<u8>) -> Result<Cancel, Error> {
        if raw.len() != 13 {
            return Err(Error::InvalidMsgLen);
        }
        Ok(Cancel {
            index: u32::from_be_bytes(raw[1..5].try_into().unwrap()),
            begin: u32::from_be_bytes(raw[5..9].try_into().unwrap()),
            len: u32::from_be_bytes(raw[9..].try_into().unwrap()),
        })
    }
    pub fn bytes(&self) -> Vec<u8> {
        let mut raw = vec![6];
        raw.extend_from_slice(&self.index.to_be_bytes());
        raw.extend_from_slice(&self.begin.to_be_bytes());
        raw.extend_from_slice(&self.len.to_be_bytes());
        return raw;
    }
}

#[derive(Debug)]
pub struct Port {
    listen_port: u16,
}

impl Port {
    pub fn try_from_bytes(raw: Vec<u8>) -> Result<Port, Error> {
        if raw.len() != 3 {
            return Err(Error::InvalidMsgLen);
        }
        Ok(Port {
            listen_port: u16::from_be_bytes(raw[1..].try_into().unwrap()),
        })
    }
    pub fn bytes(&self) -> Vec<u8> {
        let mut raw = vec![4];
        raw.extend_from_slice(&self.listen_port.to_be_bytes());
        return raw;
    }
}
