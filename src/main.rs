mod message;
mod peer;
mod peer_proto;
mod piece;

use peer_proto::PeerProto;
use rand::distributions::{Alphanumeric, DistString};
use std::sync::Arc;
use std::time::Duration;
use std::{fs, thread};

use std::fs::File;
use std::io::prelude::*;
use std::net::{IpAddr, Ipv4Addr, TcpStream, SocketAddr};

use lava_torrent::torrent::v1::Torrent;
use lava_torrent::tracker::{Peer, TrackerResponse};

use crate::message::Request;
use crate::peer_proto::Message;

trait Test {}

impl Test for Peer {}

fn main() {
    let torrent2 = Torrent::read_from_file("C:/Users/h04x/Downloads/koh2.torrent").unwrap();
    let peer_id = Alphanumeric.sample_string(&mut rand::thread_rng(), 20);

    println!(
        "torrent files total len {}",
        torrent2
            .files
            .as_ref()
            .unwrap()
            .iter()
            .map(|i| i.length)
            .sum::<i64>()
    );
    println!("pieces cnt {}", &torrent2.pieces.len());
    println!("one piece len {}", &torrent2.piece_length);
    println!(
        "piece len * piece cnt {}",
        *&torrent2.pieces.len() as i64 * &torrent2.piece_length
    );

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
    let info_hash = torrent2.info_hash_bytes();

    let mut pieces = Vec::new();
    for hash in torrent2.pieces {
        pieces.push(piece::Piece::new(
            hash.try_into().expect("some piece hash mismatch length"),
        ));
    }

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

        //let mut threads = Vec::new();
        let s = TcpStream::connect("78.61.204.38:16419".parse::<SocketAddr>().unwrap()).unwrap();

        let  pp = Arc::new(PeerProto::handshake(s, &info_hash, &peer_id.as_bytes()).unwrap());
        let  pp2 = pp.clone();
        thread::spawn(move ||{
            dbg!(pp2.recv());
        });
        dbg!(pp.recv());
        for peer in peers {
            //let p = peer::Peer::new(peer.addr, info_hash.clone(), peer_id.as_bytes().to_vec());

            //p.connect();
            //p.send();
            //let info_hash = info_hash.clone();
            //let peer_id = peer_id.clone();
            /*threads.push(thread::spawn(move || {
                match TcpStream::connect(peer.addr) {
                    Ok(s) => {
                        if s.peer_addr().unwrap().ip() == IpAddr::V4(Ipv4Addr::new(78, 61, 204, 38))
                        {
                            //s.set_read_timeout(Duration::secs(1)).u;
                            println!("connected {:?}, read timeout {:?}", s, s.read_timeout());
                            let mut pp =
                                PeerProto::handshake(s, &info_hash, &peer_id.as_bytes()).unwrap();

                            loop {
                                let msg = pp.recv().unwrap();
                                println!("msg: {:?}", msg);
                                match msg {
                                    Message::Bitfield(_) => {
                                        pp.send(Message::Interested).unwrap();

                                        //pp.send(Message::Request(Request::new(616,1,2_u32.pow(14)))).unwrap();
                                    }
                                    Message::Unchoke => {
                                        pp.send(Message::Request(Request::new(
                                            8671,
                                            1,
                                            2_u32.pow(14),
                                        )))
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
            })); */
        }

        /*for t in threads {
            t.join().unwrap();
        }*/
    }
}
