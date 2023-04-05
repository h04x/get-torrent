use thiserror::Error;

#[derive(Debug)]
pub struct Bitfield {}

impl Bitfield {
    pub fn from_bytes(raw: Vec<u8>) -> Bitfield {
        Bitfield {}
    }
    pub fn bytes(&self) -> Vec<u8> {
        unimplemented!()
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

#[derive(Debug)]
pub struct Piece {
    index: u32,
    begin: u32,
    block: Vec<u8>,
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
