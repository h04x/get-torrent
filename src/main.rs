mod message;
mod peer_proto;

use peer_proto::PeerProto;
use rand::distributions::{Alphanumeric, DistString};
use std::{fs, thread};

use std::fs::File;
use std::io::prelude::*;
use std::net::{TcpStream, IpAddr, Ipv4Addr};

use lava_torrent::torrent::v1::Torrent;
use lava_torrent::tracker::TrackerResponse;

use crate::message::Request;
use crate::peer_proto::Message;

fn main() {
    let torrent2 = Torrent::read_from_file("C:/Users/h04x/Downloads/koh2.torrent").unwrap();
    let peer_id = Alphanumeric.sample_string(&mut rand::thread_rng(), 20);

    println!("torrent files total len {}", torrent2.files.as_ref().unwrap().iter().map(|i| i.length).sum::<i64>());
    println!("pieces cnt {}", &torrent2.pieces.len());
    println!("one piece len {}", &torrent2.piece_length);
    println!("piece len * piece cnt {}", *&torrent2.pieces.len() as i64 * &torrent2.piece_length);

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
        let mut threads = Vec::new();
        for peer in peers {
            let info_hash = torrent2.clone();
            let peer_id = peer_id.clone();
            threads.push(thread::spawn(move || {
                match TcpStream::connect(peer.addr) {
                    Ok(s) => {
                        if s.peer_addr().unwrap().ip()
                            == IpAddr::V4(Ipv4Addr::new(78, 61, 204, 38))
                        {
                        println!("connected {:?}, read timeout {:?}", s, s.read_timeout());
                        let mut pp = PeerProto::handshake(
                            s,
                            &info_hash.info_hash_bytes(),
                            &peer_id.as_bytes(),
                        )
                        .unwrap();

                        loop {
                            let msg = pp.recv().unwrap();
                            println!("msg: {:?}", msg);
                            match msg {
                                Message::Bitfield(_) => {
                                    pp.send(Message::Interested).unwrap();
                                    //pp.send(Message::Request(Request::new(616,1,2_u32.pow(14)))).unwrap();
                                }
                                Message::Unchoke => {
                                    pp.send(Message::Request(Request::new(616, 1, 2_u32.pow(14))))
                                        .unwrap();
                                }
                                _ => (),
                            }
                            //let bitf = Message::new(MessageType::MsgBitfield, &[]);
                        }
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
