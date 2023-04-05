use core::fmt;
use data_encoding::HEXLOWER;
use std::io::{Read, Write};
use thiserror::Error;

use crate::message::{self, Bitfield, Cancel, Have, Piece, Port, Request};

#[derive(Clone)]
pub struct Handshake {
    pub extensinons: [u8; 8],
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
            self.extensinons,
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
            extensinons: *b"\x00\x00\x00\x00\x00\x00\x00\x00",
            info_hash: info_hash.try_into().unwrap(),
            peer_id: peer_id.try_into().unwrap(),
        })
    }

    pub fn bytes(&self) -> [u8; 68] {
        let mut bytes = [0; 68];
        bytes[0] = 19;
        bytes[1..20].copy_from_slice(b"BitTorrent protocol");
        bytes[20..28].copy_from_slice(&self.extensinons);
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
            extensinons: bytes[20..28].try_into().unwrap(),
            info_hash: bytes[28..48].try_into().unwrap(),
            peer_id: bytes[48..68].try_into().unwrap(),
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MessageType {
    MsgChoke,
    MsgUnchoke,
    MsgInterested,
    MsgNotInterested,
    MsgHave,
    MsgBitfield,
    MsgRequest,
    MsgPiece,
    MsgCancel,
}

impl TryFrom<u8> for MessageType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MessageType::MsgChoke),
            1 => Ok(MessageType::MsgUnchoke),
            2 => Ok(MessageType::MsgInterested),
            3 => Ok(MessageType::MsgNotInterested),
            4 => Ok(MessageType::MsgHave),
            5 => Ok(MessageType::MsgBitfield),
            6 => Ok(MessageType::MsgRequest),
            7 => Ok(MessageType::MsgPiece),
            8 => Ok(MessageType::MsgCancel),
            _ => Err(()),
        }
    }
}

/*impl From<MessageType> for u8 {
    fn from(value: MessageType) -> Self {
        value as u8
    }
}*/

/*#[derive(Debug)]
pub struct Message {
    pub id: MessageType,
    pub payload: Vec<u8>,
}*/

/*#[derive(Debug)]
pub enum MessageError {
    PacketLenLessThanFiveBytes,
    PacketLenLessThanDeclared,
    UnknownMessageType,
}*/

/*impl Message {
    pub fn new(msg_id: MessageType, payload: &[u8]) -> Message {
        Message {
            id: msg_id,
            payload: payload.to_vec(),
        }
    }
    pub fn bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        let len = self.payload.len() as u32 + 1;
        bytes.extend_from_slice(&len.to_be_bytes());
        bytes.push(self.id as u8);
        bytes.extend_from_slice(&self.payload);
        bytes
    }
    pub fn from_bytes(bytes: &[u8]) -> Result<Message, MessageError> {
        if bytes.len() < 5 {
            return Err(MessageError::PacketLenLessThanFiveBytes);
        }
        let len = u32::from_be_bytes(bytes[0..4].try_into().unwrap());
        if len > bytes.len() as u32 {
            return Err(MessageError::PacketLenLessThanDeclared);
        }
        Ok(Message {
            id: bytes[4]
                .try_into()
                .map_err(|_| MessageError::UnknownMessageType)?,
            payload: bytes[0..len as usize].to_vec(),
        })
    }
}*/

pub struct PeerProto<S: Read + Write> {
    stream: S,
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
    #[error("Packet len less than 4 bytes")]
    PktLenLessThanFourBytes,
    #[error("Message len less than 1 bytes")]
    MsgLenLessThanOneBytes,
    #[error("Message error")]
    Msg(#[from] MessageError),
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
    Unknown(Vec<u8>),
    KeepAlive,
}

impl Message {
    fn try_from_bytes(raw: Vec<u8>) -> Result<Message, MessageError> {
        return Ok(match raw.get(0) {
            Some(0) => Self::Choke,
            Some(1) => Self::Unchoke,
            Some(2) => Self::Interested,
            Some(3) => Self::NotInterested,
            Some(4) => Self::Have(Have::try_from_bytes(raw)?),
            Some(5) => Self::Bitfield(Bitfield::from_bytes(raw)), //
            Some(6) => Self::Request(Request::try_from_bytes(raw)?),
            Some(7) => Self::Piece(Piece::try_from_bytes(raw)?),
            Some(8) => Self::Cancel(Cancel::try_from_bytes(raw)?),
            Some(9) => Self::Port(Port::try_from_bytes(raw)?),
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
            Self::Have(h) => h.bytes(),
            Self::Bitfield(b) => b.bytes(),
            Self::Request(r) => r.bytes(),
            Self::Piece(p) => p.bytes(),
            Self::Cancel(c) => c.bytes(),
            Self::Port(p) => p.bytes(),
            Self::Unknown(raw) => raw,
            Self::KeepAlive => vec![],
        }
    }
}

impl<S: Read + Write> PeerProto<S> {
    pub fn handshake<'a, 'b>(
        mut stream: S,
        info_hash: &'a [u8],
        peer_id: &'b [u8],
    ) -> Result<PeerProto<S>, Error> {
        let hs = Handshake::build(info_hash, peer_id)?;
        stream.write(&hs.bytes())?;

        let mut hs_buf = [0; 68];
        let len = stream.read(&mut hs_buf)?;
        if len == 0 {
            return Err(Error::ConnectionClosed);
        }
        let peer_hs = Handshake::from_bytes(&hs_buf)?;

        if hs.info_hash != peer_hs.info_hash {
            return Err(Error::PeerInfoHashNotEq);
        }
        Ok(PeerProto { stream: stream })
    }

    pub fn recv(&mut self) -> Result<Message, RecvMsgError> {
        let mut head = [0; 4];
        let plen = self.stream.read(&mut head)?;
        if plen < 4 {
            return Err(RecvMsgError::PktLenLessThanFourBytes);
        };
        let mlen = u32::from_be_bytes(head[0..4].try_into().unwrap()) as usize;
        let mut msg_buf = vec![0u8; mlen];
        let mut pulled_bytes = 0;
        while pulled_bytes < mlen {
            let plen = self.stream.read(&mut msg_buf[pulled_bytes..])?;
            pulled_bytes += plen;
        }
        Ok(Message::try_from_bytes(msg_buf)?)
    }

    pub fn send(&mut self, msg: Message) -> std::io::Result<usize> {
        let msg = msg.bytes();
        let head = (msg.len() as u32).to_be_bytes();
        let mut raw = Vec::new();
        raw.extend_from_slice(&head);
        raw.extend_from_slice(&msg);
        println!("{:?}", raw);
        self.stream.write(&raw)
    }
}
