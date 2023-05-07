use std::{net::SocketAddr, thread};

use bittorrent_peer_proto::message;
use crossbeam_channel::{Receiver, Sender};

pub struct DhtDispatch {
    pub msg_port_recv: Receiver<(SocketAddr, message::Port)>,
    pub msg_port_send: Sender<(SocketAddr, message::Port)>,
    pub info_hash: [u8; 20],
}

impl DhtDispatch {
    pub fn new(info_hash: [u8; 20]) -> DhtDispatch {
        let (msg_port_send, msg_port_recv) = crossbeam_channel::unbounded();
        DhtDispatch {
            msg_port_recv,
            msg_port_send,
            info_hash,
        }
    }

    pub fn run(&self, send_peer: Sender<SocketAddr>) {
        let info_hash = self.info_hash.clone();
        let msg_port_recv = self.msg_port_recv.clone();
        thread::spawn(move || Self::worker(info_hash, send_peer, msg_port_recv));
    }

    fn worker(
        info_hash: [u8; 20],
        send_peer: Sender<SocketAddr>,
        msg_port_recv: Receiver<(SocketAddr, message::Port)>,
        ) {
        if let Ok(peers) = dht_get_peers::get_peers(info_hash) {
            for peer in peers {
                send_peer.send(peer);
            }
        }

        let mut port_msgs = Vec::new();
        while let Ok(port_msg) = msg_port_recv.recv() {
            port_msgs.push(port_msg);
            if port_msgs.len() > 5 {
                let port_msgs = port_msgs.iter().map(|pm| 
                    SocketAddr::new(pm.0.ip(), pm.1.listen_port)
                ).collect::<Vec<_>>();
                if let Ok(peers) = dht_get_peers::get_peers_bs(info_hash, port_msgs.as_slice()) {
                    for peer in peers {
                        send_peer.send(peer);
                    }                    
                }                
            }
            port_msgs.clear();
        }

        //}
    }
}
