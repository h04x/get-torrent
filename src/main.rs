const BLOCK_SIZE: u32 = 2u32.pow(14);
const NAME: &str = "get-torrent";
const UT_PEX_EXTENDED_MSG_ID: u8 = 1;
const PARALLEL_REQUEST_PER_PEER: usize = 4;

mod dht_dispatch;
mod peer_dispatch;
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

use crate::dht_dispatch::DhtDispatch;
use crate::peer_dispatch::PeerDispatch;
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

    if true {
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
    let info_hash = <[u8; 20]>::try_from(torrent.info_hash_bytes()).unwrap();

    let piece_dispatch = PieceDispatch::new(&torrent);
    let dht_dispatch = DhtDispatch::new(info_hash);
    let peer_dispatch = PeerDispatch::run(
        &resp,
        info_hash,
        peer_id.into_bytes().try_into().unwrap(),
        piece_dispatch.rx,
        piece_dispatch.tx,
        piece_dispatch.complete_piece.clone(),
        dht_dispatch.msg_port_send.clone(),
    )
    .unwrap();

    dht_dispatch.run(peer_dispatch.send_peer);

    loop {
        thread::sleep(Duration::from_secs(1));
        println!(
            "active peers: {:?}, complete pieces: {}/{}",
            peer_dispatch.active_peers.lock().len(),
            piece_dispatch.complete_piece.lock().len(),
            torrent.pieces.len()
        );
    }
}
