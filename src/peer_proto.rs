use core::fmt;
use data_encoding::HEXLOWER;
use std::io::{Read, Write};
use thiserror::Error;

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
    tmp_buf: [u8; 1024],
    //pub msg_buf: Vec<u8>
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
    ConnectionClosed
}

#[derive(Error, Debug)]
pub enum ReadMsgError {
    #[error("Error while io")]
    Io(#[from] std::io::Error),
    #[error("Packet len less than 4 bytes")]
    PktLenLessThanFourBytes,
    #[error("Message len less than 1 bytes")]
    MsgLenLessThanOneBytes,
    #[error("Message buffer overflow")]
    MsgBufOverflow,
    #[error("Infinite read loop")]
    InfiniteReadLoop,
}



#[derive(Debug)]
pub enum Message {
    BitBlt(())
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
        //println!("plen {}, buf {:?}", len, &hsbuf);

        if hs.info_hash != peer_hs.info_hash {
            return Err(Error::PeerInfoHashNotEq);
        }
        //let msg_buf = read_exact_msg_len(&mut self.stream, &mut self.tmp_buf).unwrap();
        //println!("{:?}", msg_buf);
        Ok(PeerProto {
            stream: stream,
            tmp_buf: [0; 1024],
        })
    }

    pub fn read_msg(&mut self) -> Result<Message, ReadMsgError> {
        let mut head = [0; 4];
        let plen = self.stream.read(&mut head)?;
        //println!("{:?}", head);
        if plen < 4 {
            return Err(ReadMsgError::PktLenLessThanFourBytes);
        };
        let mlen = u32::from_be_bytes(head[0..4].try_into().unwrap()) as usize;
        if mlen < 1 {
            return Err(ReadMsgError::MsgLenLessThanOneBytes);
        };
        let mut msg_buf = vec![0u8; mlen];
        //msg_buf[0..5].copy_from_slice(&head);
        let mut pulled_bytes = 0;
        while pulled_bytes < mlen {
            let plen = self.stream.read(&mut msg_buf[pulled_bytes..])?;
            pulled_bytes += plen;
            //println!("mlen {} plen {} pulled {:?}", mlen, plen, pulled_bytes);
        }
        //Ok(msg_buf)
        Ok(Message::BitBlt(()))
    }
}

/*fn read_exact_msg_len<S: Read>(
    stream: &mut S,
    tmpbuf: &mut [u8],
) -> Result<Vec<u8>, ReadMsgError> {
    let plen = stream.read(tmpbuf)?;
    if plen < 5 {
        return Err(ReadMsgError::MsgLenLessThanOneBytes);
    }
    let mlen = u32::from_le_bytes(tmpbuf[0..4].try_into().unwrap()) as usize;
    let mut buf = Vec::new();

    println!(
        "mlen {}, plen {}, packet {:?}",
        mlen,
        plen,
        &tmpbuf[0..plen]
    );

    buf.extend_from_slice(&tmpbuf[0..plen]);

    let mut limit = 0;
    while buf.len() != mlen {
        let plen = stream.read(tmpbuf)?;
        // prevent buf blobing
        if buf.len() > tmpbuf.len() * 10 {
            return Err(ReadMsgError::MsgBufOverflow);
        }
        // prevent infinite read
        if limit > 20 {
            return Err(ReadMsgError::InfiniteReadLoop);
        }
        buf.extend_from_slice(&tmpbuf[0..plen]);
        println!(
            "mlen {}, plen, {}, buf.len {}, packet {:?}",
            mlen,
            plen,
            buf.len(),
            &tmpbuf[0..plen]
        );
        limit += 1;
    }
    Ok(buf)
}

pub trait Test {

}
#[derive(Debug)]
struct One {

}

impl Test for One {

}
#[derive(Debug)]
struct Two {

}

impl Test for Two {

}*/