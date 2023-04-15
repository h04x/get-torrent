mod message;
mod peer;
mod peer_proto;
mod piece;

use peer_proto::PeerProto;
use rand::distributions::{Alphanumeric, DistString};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{fs, thread};

use std::fs::File;
use std::io::prelude::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};

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
        let mut peers_data = Arc::new(Mutex::new(HashMap::new()));
        for peer in peers {
            let info_hash = info_hash.clone();
            peer::Peer::start_receiver(
                peer.addr,
                info_hash,
                peer_id.as_bytes().to_vec(),
                peers_data.clone(),
            );
        }
        thread::sleep(Duration::from_secs(1));
        peers_data.lock().unwrap().iter().next().unwrap().1.proto.send(Message::Request(message::Request::new(0, 0, 2u32.pow(14))));

        loop {
            thread::sleep(Duration::from_secs(1));
            println!("peers.len() {:?}", peers_data.lock().unwrap().len());
        }
    }
}
