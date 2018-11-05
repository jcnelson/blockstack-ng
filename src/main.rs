extern crate rand;
extern crate bitcoin;
extern crate ini;
#[macro_use] extern crate log; 

mod burnchains;
mod util;

use burnchains::indexer::BurnchainIndexer;
use burnchains::bitcoin::Error as net_error;
use util::log as logger;

fn main() {
    logger::init().unwrap();

    let mut bitcoin_indexer = burnchains::bitcoin::indexer::BitcoinIndexer::new();
    bitcoin_indexer.setup("/tmp/test-blockstack-ng").unwrap();

    let mut do_handshake = true;

    loop {
        if do_handshake {
            let handshake_result = bitcoin_indexer.connect_handshake_backoff("mainnet");
            match handshake_result {
                Ok(()) => {
                    // connection established!
                    do_handshake = false;
                }
                Err(_) => {
                    // need to try again 
                    continue;
                }
            }
        }

        let msg_result = bitcoin_indexer.recv_message();
        match msg_result {
            Ok(msg) => {
                // got a message, so handle it!
                let handled = bitcoin_indexer.handle_message(&msg);
                match handled {
                    Ok(()) => {},
                    Err(net_error::UnhandledMessage) => {
                        debug!("Unhandled message {:?}", msg);
                    }
                    Err(net_error::ConnectionBroken) => {
                        debug!("Re-establish peer connection");
                        do_handshake = true;
                    }
                    Err(e) => {
                        panic!("Unhandled error while handling {:?}: {:?}", msg, e);
                    }
                }
            }
            Err(net_error::ConnectionBroken) => {
                debug!("Re-establish peer connection");
                do_handshake = true;
            }
            Err(e) => {
                panic!("Unhandled error while receiving a message: {:?}", e);
            }
        }
    }
}
