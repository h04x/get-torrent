const BLOCK_SIZE: u32 = 2u32.pow(14);
const NAME: &str = "torrent-test";
const UT_PEX_EXTENDED_MSG_ID: u8 = 1;
const PARALLEL_REQUEST_PER_PEER: usize = 4;

mod peer;
mod peer_proto;
mod piece;
mod piece_dispatch;
#[cfg(test)]
mod tests;

use parking_lot::Mutex;
use rand::distributions::{Alphanumeric, DistString};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use std::{env, fs, thread};

use std::fs::File;
use std::io::prelude::*;

use lava_torrent::torrent::v1::Torrent;
use lava_torrent::tracker::{Peer, TrackerResponse};

use crate::piece_dispatch::PieceDispatch;

trait Test {}

impl Test for Peer {}

fn main() {
    let torrent = Torrent::read_from_file(
        env::args()
            .nth(1)
            .unwrap_or("torrent/debian.iso.torrent".to_string()),
    )
    .unwrap();
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
    println!("torrent size: {}", torrent.length);
    println!("pieces count {}", torrent.pieces.len());
    println!("one piece length {}", &torrent.piece_length);

    if false {
        let info_hash = urlencoding::encode_binary(&torrent.info_hash_bytes()).into_owned();
        let params = [
            ("peer_id", peer_id.as_str()),
            ("port", "6888"),
            ("downloaded", "0"),
            ("uploaded", "0"),
            ("left", "0"),
            ("event", "started"),
        ];

        let announce = torrent.announce.as_ref().unwrap();
        let url = reqwest::Url::parse_with_params(announce, &params).unwrap();
        let url = reqwest::Url::parse(&format!("{}&info_hash={}", url, info_hash)).unwrap();

        let client = reqwest::blocking::Client::builder()
            .user_agent(NAME)
            .build()
            .unwrap();
        let resp = client.get(url).send().unwrap();
        let mut f = File::create("peers.torrent").unwrap();
        f.write_all(&resp.bytes().unwrap()).unwrap();
    }

    let s = fs::read("peers.torrent").unwrap();
    let resp = TrackerResponse::from_bytes(s).unwrap();
    let info_hash = torrent.info_hash_bytes();

    let piece_dispatch = PieceDispatch::new(&torrent);

    if let TrackerResponse::Success {
        interval,
        min_interval,
        peers,
        ..
    } = resp
    {
        println!("interval:{:?}, min_interval {:?}", interval, min_interval);
        let peers_data = Arc::new(Mutex::new(HashMap::new()));
        let complete_pieces = Arc::new(Mutex::new(Vec::new()));

        //let pieces = Arc::new(Mutex::new(pieces));
        //let chan_tx = start_piece_receiver(peers_data.clone(), pieces.clone());

        for peer in peers {
            let info_hash = info_hash.clone();
            peer::Peer::start_receiver(
                peer.addr,
                info_hash,
                peer_id.as_bytes().to_vec(),
                peers_data.clone(),
                complete_pieces.clone(),
                peer, //chan_tx.clone(),
                piece_dispatch.rx.clone(),
                piece_dispatch.tx.clone(),
            );
            //break;
        }

        loop {
            thread::sleep(Duration::from_secs(1));
            println!(
                "active peers: {:?}, complete pieces: {}/{}",
                peers_data.lock().len(),
                complete_pieces.lock().len(),
                torrent.pieces.len()
            );
        }
    }
}
