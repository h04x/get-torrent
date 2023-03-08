mod torrent_file;

use crate::torrent_file::MetaInfo;
use bendy::decoding::FromBencode;

use rand::distributions::{Alphanumeric, DistString};
use std::fs;

use lava_torrent::torrent::v1::Torrent;

fn main() {
    let s = fs::read("C:/Users/h04x/Downloads/koh2.torrent").unwrap();
    let torrent = MetaInfo::from_bencode(&s).unwrap();

    let torrent2 = Torrent::read_from_file("C:/Users/h04x/Downloads/koh2.torrent").unwrap();
    println!("{}", torrent2.info_hash());

    let peer_id = Alphanumeric.sample_string(&mut rand::thread_rng(), 20);
    println!("{}", peer_id);
    println!("{:?}", &torrent.info_hash);

    let info_hash = urlencoding::encode_binary(&torrent.info_hash).into_owned();
    dbg!(&info_hash);
    let params = [
        //("info_hash", "%%25"),
        ("peer_id", peer_id.as_str()),
        ("port", "6888"),
        ("downloaded", "0"),
        ("uploaded", "0"),
        ("left", "0"),
        ("event", "started")
    ];

    let mut url = reqwest::Url::parse_with_params(&torrent.announce, &params).unwrap();
    let url = reqwest::Url::parse(&format!("{}&info_hash={}",url, info_hash)).unwrap();

    // url.query_pairs_mut().extend_pairs(params2);
    //let url = url.join(info_hash.as_str()).unwrap();
    //println!("{:#?}", url);
    let client = reqwest::blocking::Client::builder()
        .user_agent("torrent-test")
        .build().unwrap();
    //let resp = client.get(url).send().unwrap();
    //.json::<HashMap<String, String>>()?;
    //println!("{:#?}", resp.text());
    //Ok(())
}
