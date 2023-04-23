const BLOCK_SIZE: u32 = 2u32.pow(14);

mod message;
mod peer;
mod peer_proto;
mod piece;

use parking_lot::Mutex;
use rand::distributions::{Alphanumeric, DistString};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use std::{fs, thread};

use std::fs::File;
use std::io::prelude::*;

use lava_torrent::torrent::v1::Torrent;
use lava_torrent::tracker::{Peer, TrackerResponse};

use crate::piece::start_piece_receiver;

trait Test {}

impl Test for Peer {}

fn main() {
    let torrent_file = Torrent::read_from_file("C:/Users/h04x/Downloads/1file.torrent").unwrap();
    let peer_id = Alphanumeric.sample_string(&mut rand::thread_rng(), 20);

    /*println!(
        "torrent files total len {}",
        torrent2
            .files
            .as_ref()
            .unwrap()
            .iter()
            .map(|i| i.length)
            .sum::<i64>()
    );*/
    println!("torrent size: {}", torrent_file.length);
    println!("pieces count {}", torrent_file.pieces.len());
    println!("one piece length {}", &torrent_file.piece_length);

    if false {
        let info_hash = urlencoding::encode_binary(&torrent_file.info_hash_bytes()).into_owned();
        let params = [
            ("peer_id", peer_id.as_str()),
            ("port", "6888"),
            ("downloaded", "0"),
            ("uploaded", "0"),
            ("left", "0"),
            ("event", "started"),
        ];

        let announce = torrent_file.announce.as_ref().unwrap();
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
    let info_hash = torrent_file.info_hash_bytes();

    let mut pieces = Vec::new();
    let mut len = 0u32;
    for (index, hash) in torrent_file.pieces.iter().enumerate() {
        len = torrent_file.piece_length as u32;
        if index == torrent_file.pieces.len() - 1 {
            len = (torrent_file.length % torrent_file.piece_length) as u32
        }
        pieces.push(piece::Piece::new(
            hash.clone().try_into().expect("piece hash mismatch length"),
            len,
        ));
    }

    if let TrackerResponse::Success {
        interval,
        min_interval,
        peers,
        ..
    } = resp
    {
        println!("interval:{:?}, min_interval {:?}", interval, min_interval);
        let peers_data = Arc::new(Mutex::new(HashMap::new()));

        let chan_tx = start_piece_receiver(peers_data.clone(), pieces);

        for peer in peers {
            let info_hash = info_hash.clone();
            peer::Peer::start_receiver(
                peer.addr,
                info_hash,
                peer_id.as_bytes().to_vec(),
                peers_data.clone(),
                chan_tx.clone(),
            );
        }

        loop {
            thread::sleep(Duration::from_secs(1));
            println!("peers.len() {:?}", peers_data.lock().len());
        }
    }
}
