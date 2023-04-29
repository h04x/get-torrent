use core::fmt;
use data_encoding::HEXLOWER;
use std::{
    io::{Read, Write},
    net::TcpStream,
};
use thiserror::Error;

use crate::peer_proto::message::{self, Bitfield, Cancel, Extended, Have, Piece, Port, Request};

use message::prepend;

#[derive(Clone)]
pub struct Handshake {
    pub extensions: [u8; 8],
    pub info_hash: [u8; 20],
    pub peer_id: [u8; 20],
}

#[derive(Clone, Debug, Error)]
pub enum HandshakeError {
    #[error("InfoHash must be 20 bytes len")]
    InfoHashMismatchLen,
    #[error("PeerId must be 20 bytes len")]
    PeerIdMismatchLen,
    #[error("Packet len must be 68")]
    PacketMismatchLen,
    #[error("First byte must be 19")]
    FirstByteMismatchNineteen,
    #[error("Proto name must be 'BitTorrent protocol'")]
    ProtoNameMismatch,
}

impl fmt::Debug for Handshake {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ext: {:?}, infohash: {}, peer: {:#?}",
            self.extensions,
            HEXLOWER.encode(&self.info_hash),
            String::from_utf8_lossy(&self.peer_id)
        )
    }
}

impl Handshake {
    pub fn build<'a>(info_hash: &'a [u8], peer_id: &'a [u8]) -> Result<Handshake, HandshakeError> {
        if info_hash.len() != 20 {
            return Err(HandshakeError::InfoHashMismatchLen);
        }
        if peer_id.len() != 20 {
            return Err(HandshakeError::PeerIdMismatchLen);
        }
        Ok(Handshake {
            extensions: *b"\x00\x00\x00\x00\x00\x10\x00\x00",
            info_hash: info_hash.try_into().unwrap(),
            peer_id: peer_id.try_into().unwrap(),
        })
    }

    pub fn bytes(&self) -> [u8; 68] {
        let mut bytes = [0; 68];
        bytes[0] = 19;
        bytes[1..20].copy_from_slice(b"BitTorrent protocol");
        bytes[20..28].copy_from_slice(&self.extensions);
        bytes[28..48].copy_from_slice(&self.info_hash);
        bytes[48..68].copy_from_slice(&self.peer_id);
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Handshake, HandshakeError> {
        //println!("{}", bytes.len());
        if bytes.len() != 68 {
            return Err(HandshakeError::PacketMismatchLen);
        }
        if bytes[0] != 19 {
            return Err(HandshakeError::FirstByteMismatchNineteen);
        }
        if &bytes[1..20] != b"BitTorrent protocol" {
            return Err(HandshakeError::ProtoNameMismatch);
        }
        Ok(Handshake {
            extensions: bytes[20..28].try_into().unwrap(),
            info_hash: bytes[28..48].try_into().unwrap(),
            peer_id: bytes[48..68].try_into().unwrap(),
        })
    }

    pub fn ut_pex_support(&self) -> bool {
        (self.extensions[5] & 0x10) > 0
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error while handshake")]
    Handshake(#[from] HandshakeError),
    #[error("Error while io")]
    Io(#[from] std::io::Error),
    #[error("Our and peer info_hash not equal")]
    PeerInfoHashNotEq,
    #[error("Remote peer closed connection")]
    ConnectionClosed,
}

#[derive(Error, Debug)]
pub enum RecvMsgError {
    #[error("Error while io")]
    Io(#[from] std::io::Error),
    #[error("Message error")]
    Msg(#[from] MessageError),
    #[error("Remote peer closed connection")]
    ConnectionClosed,
}

#[derive(Error, Debug)]
pub enum MessageError {
    #[error("Message error")]
    MsgError(#[from] message::Error),
}

#[derive(Debug)]
pub enum Message {
    Choke,
    Unchoke,
    Interested,
    NotInterested,
    Have(Have),
    Bitfield(Bitfield),
    Request(Request),
    Piece(Piece),
    Cancel(Cancel),
    Port(Port),
    Extended(Extended),
    Unknown(Vec<u8>),
    KeepAlive,
}

impl Message {
    fn try_from_bytes(raw: Vec<u8>) -> Result<Message, MessageError> {
        return Ok(match raw.first() {
            Some(0) => Self::Choke,
            Some(1) => Self::Unchoke,
            Some(2) => Self::Interested,
            Some(3) => Self::NotInterested,
            Some(4) => Self::Have(Have::try_from_bytes(&raw[1..])?),
            Some(5) => Self::Bitfield(Bitfield::try_from_bytes(&raw[1..])?),
            Some(6) => Self::Request(Request::try_from_bytes(&raw[1..])?),
            Some(7) => Self::Piece(Piece::try_from_bytes(&raw[1..])?),
            Some(8) => Self::Cancel(Cancel::try_from_bytes(&raw[1..])?),
            Some(9) => Self::Port(Port::try_from_bytes(&raw[1..])?),
            Some(20) => Self::Extended(Extended::try_from_bytes(&raw[1..])?),
            Some(_) => Self::Unknown(raw),
            None => Self::KeepAlive,
        });
    }

    fn bytes(self) -> Vec<u8> {
        match self {
            Self::Choke => vec![0],
            Self::Unchoke => vec![1],
            Self::Interested => vec![2],
            Self::NotInterested => vec![3],
            Self::Have(h) => prepend(&[4], &h.bytes()),
            Self::Bitfield(b) => prepend(&[5], &b.bytes()),
            Self::Request(r) => prepend(&[6], &r.bytes()),
            Self::Piece(p) => prepend(&[7], &p.bytes()),
            Self::Cancel(c) => prepend(&[8], &c.bytes()),
            Self::Port(p) => prepend(&[9], &p.bytes()),
            Self::Extended(e) => prepend(&[20], &e.bytes()),
            Self::Unknown(raw) => raw,
            Self::KeepAlive => vec![],
        }
    }
}

pub struct PeerProto {
    pub stream: TcpStream,
    pub peer_handshake: Handshake,
}

impl PeerProto {
    pub fn handshake(
        mut stream: TcpStream,
        info_hash: &[u8],
        peer_id: &[u8],
    ) -> Result<PeerProto, Error> {
        let hs = Handshake::build(info_hash, peer_id)?;
        stream.write_all(&hs.bytes())?;

        let mut hs_buf = [0; 68];
        let len = stream.read(&mut hs_buf)?;
        if len == 0 {
            return Err(Error::ConnectionClosed);
        }
        let peer_handshake = Handshake::from_bytes(&hs_buf)?;

        if hs.info_hash != peer_handshake.info_hash {
            return Err(Error::PeerInfoHashNotEq);
        }

        let pp = PeerProto {
            stream,
            peer_handshake,
        };

        if pp.peer_handshake.ut_pex_support() {
            pp.ut_pex_handshake();
            // await extension message TODO: replace await extension to parse
            pp.recv();
        }

        Ok(pp)
    }

    pub fn ut_pex_handshake(&self) {
        if self.peer_handshake.ut_pex_support() {
            self.send(Message::Extended(Extended::handshake()));
        }
    }

    pub fn recv(&self) -> Result<Message, RecvMsgError> {
        let mut head = [0; 4];
        let plen = (&self.stream).read(&mut head)?;
        if plen == 0 {
            return Err(RecvMsgError::ConnectionClosed);
        };
        if plen < 4 {
            return Err(RecvMsgError::Msg(MessageError::MsgError(
                message::Error::InvalidMsgLen,
            )));
        };
        let mlen = u32::from_be_bytes(head[0..4].try_into().unwrap()) as usize;
        let mut msg_buf = vec![0u8; mlen];
        let mut pulled_bytes = 0;
        while pulled_bytes < mlen {
            let plen = (&self.stream).read(&mut msg_buf[pulled_bytes..])?;
            pulled_bytes += plen;
        }
        Ok(Message::try_from_bytes(msg_buf)?)
    }

    pub fn send(&self, msg: Message) -> std::io::Result<usize> {
        let msg = msg.bytes();
        let head = (msg.len() as u32).to_be_bytes();
        let mut raw = Vec::new();
        raw.extend_from_slice(&head);
        raw.extend_from_slice(&msg);
        (&self.stream).write(&raw)
    }
}
