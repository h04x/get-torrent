mod peer_proto;
mod message;

use peer_proto::PeerProto;
use rand::distributions::{Alphanumeric, DistString};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::{fs, thread};

use std::fs::File;
use std::io::prelude::*;
use std::net::{IpAddr, Ipv4Addr, TcpStream};

use lava_torrent::torrent::v1::Torrent;
use lava_torrent::tracker::TrackerResponse;

use crate::peer_proto::{Handshake, Message, MessageType};

fn main() {
    let torrent2 = Torrent::read_from_file("C:/Users/h04x/Downloads/koh2.torrent").unwrap();
    let peer_id = Alphanumeric.sample_string(&mut rand::thread_rng(), 20);

    if false {
        let info_hash = urlencoding::encode_binary(&torrent2.info_hash_bytes()).into_owned();
        let params = [
            ("peer_id", peer_id.as_str()),
            ("port", "6888"),
            ("downloaded", "0"),
            ("uploaded", "0"),
            ("left", "0"),
            ("event", "started"),
        ];

        let announce = torrent2.announce.as_ref().unwrap();
        let url = reqwest::Url::parse_with_params(announce, &params).unwrap();
        let url = reqwest::Url::parse(&format!("{}&info_hash={}", url, info_hash)).unwrap();

        let client = reqwest::blocking::Client::builder()
            .user_agent("torrent-test")
            .build()
            .unwrap();
        let resp = client.get(url).send().unwrap();
        let mut f = File::create("peers.torrent").unwrap();
        f.write_all(&resp.bytes().unwrap()).unwrap();
    }

    let s = fs::read("peers.torrent").unwrap();
    let resp = TrackerResponse::from_bytes(s).unwrap();
    //let resp = TrackerResponse::from_bytes(resp.bytes().unwrap());
    //let resp = BencodeElem::from_bytes(resp.bytes().unwrap()).unwrap();
    //println!("{:#?}", resp);

    if let TrackerResponse::Success {
        interval,
        peers,
        warning,
        min_interval,
        tracker_id,
        complete,
        incomplete,
        extra_fields,
    } = resp
    {
        //let handshake = Handshake::build(&torrent2.info_hash_bytes(), peer_id.as_bytes()).unwrap();

        let mut threads = Vec::new();
        for peer in peers {
            //let hs = handshake.clone().bytes();
            let info_hash = torrent2.clone();
            let peer_id = peer_id.clone();
            threads.push(thread::spawn(move || {
                match TcpStream::connect(peer.addr) {
                    Ok(mut s) => {
                        if s.peer_addr().unwrap().ip() == IpAddr::V4(Ipv4Addr::new(37, 23, 162, 7))
                        {
                            println!("connected {:?}, read timeout {:?}", s, s.read_timeout());
                            let mut pp = PeerProto::handshake(
                                s,
                                &info_hash.info_hash_bytes(),
                                &peer_id.as_bytes(),
                            )
                            .unwrap();

                            while true {
                                let m = pp.read_msg().unwrap();
                                println!("msg: {:?}", m);
                                //let bitf = Message::new(MessageType::MsgBitfield, &[]);
                            }

                            /*let mut buf = [0; 68];
                            s.write(&hs).unwrap();
                            let len = s.read(&mut buf).unwrap();
                            if let Ok(ret) = Handshake::from_bytes(&buf[0..len]) {
                                println!("{:?}, {:?}", s.peer_addr(), ret);
                                //s.write(b"\x00\x00\x00\x01\x05").unwrap();
                                //let bitf = Message::new(MessageType::MsgBitfield, &[]);
                                //println!("{:?}", bitf.bytes());
                                //s.write(&bitf.bytes()).unwrap();
                                let mut buf = [0; 256];

                                let len = s.read(&mut buf).unwrap();
                                let msg = Message::from_bytes(&buf);
                                println!("len: {}, {:?}", len, msg);
                                let int = Message::new(MessageType::MsgInterested, &[]);
                                //s.write(&int.bytes()).unwrap();
                                let mut buf = [0; 256];
                                let len = s.read(&mut buf).unwrap();
                                //let msg = Message::from_bytes(&buf);
                                println!("len: {}, {:?}", len, buf);
                                let len = s.read(&mut buf).unwrap();
                                //let msg = Message::from_bytes(&buf);
                                println!("len: {}, {:?}", len, buf);
                                let len = s.read(&mut buf).unwrap();
                                //let msg = Message::from_bytes(&buf);
                                println!("len: {}, {:?}", len, buf);*/
                        }
                    }
                    Err(_) => (),
                };
            }));
        }

        for t in threads {
            t.join().unwrap();
        }
    }
}
