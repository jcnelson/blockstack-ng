/*
    Blockstack
    ~~~~~
    copyright: (c) 2014-2015 by Halfmoon Labs, Inc.
    copyright: (c) 2016-2018 by Blockstack.org

    This file is part of Blockstack

    Blockstack is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Blockstack is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.
    You should have received a copy of the GNU General Public License
    along with Blockstack. If not, see <http://www.gnu.org/licenses/>.
*/

use std::env;
use std::net;
use std::sync::{Arc, Mutex, LockResult, MutexGuard};
use rand::{Rng, thread_rng};
use std::path::{PathBuf};

use ini::Ini;
use burnchains::indexer::*;
use burnchains::bitcoin::spv::*;

use bitcoin::network::constants as bitcoin_constants;

pub const USER_AGENT: &'static str = "Blockstack Core v21";

pub const BITCOIN_MAINNET: u32 = 0xD9B4BEF9;
pub const BITCOIN_TESTNET: u32 = 0x0709110B;
pub const BITCOIN_REGTEST: u32 = 0xDAB5BFFA;

#[derive(Debug)]
pub struct BitcoinIndexerConfig {
    // config fields
    pub peer_host: String,
    pub peer_port: u16,
    pub rpc_port: u16,
    pub username: String,
    pub password: String,
    pub timeout: u32,
    pub spv_headers_path: String
}

pub struct BitcoinIndexerRuntime {
    sock: Arc<Mutex<Option<net::TcpStream>>>,
    pub services: u64,
    pub user_agent: String,
    pub version_nonce: u64,
    pub magic: u32
}

pub struct BitcoinIndexer {
    pub config: BitcoinIndexerConfig,
    pub runtime: BitcoinIndexerRuntime
}


impl BitcoinIndexerConfig {
    fn default() -> BitcoinIndexerConfig {
        let mut spv_headers_path = env::home_dir().unwrap();
        spv_headers_path.push(".blockstack-core");
        spv_headers_path.push("bitcoin-spv-headers.dat");

        return BitcoinIndexerConfig {
            peer_host: "bitcoin.blockstack.com".to_string(),
            peer_port: 8332,
            rpc_port: 8333,
            username: "blockstack".to_string(),
            password: "blockstacksystem".to_string(),
            timeout: 30,
            spv_headers_path: spv_headers_path.to_str().unwrap().to_string()
        };
    }

    fn from_file(path: &str) -> Result<BitcoinIndexerConfig, &'static str> {
       let conf_path = PathBuf::from(path);
       if !conf_path.is_file() {
           return Err("Failed to load BitcoinIndexerConfig file: No such file or directory");
       }
       let default_config = BitcoinIndexerConfig::default();

       match Ini::load_from_file(path) {
           Ok(ini_file) => {
               // got data!
               let bitcoin_section_opt = ini_file.section(Some("bitcoin").to_owned());
               if None == bitcoin_section_opt {
                   return Err("No [bitcoin] section in config file");
               }

               let bitcoin_section = bitcoin_section_opt.unwrap();

               // defaults
               let peer_host = bitcoin_section.get("server")
                                              .unwrap_or(&default_config.peer_host);

               let peer_port = bitcoin_section.get("p2p_port")
                                              .unwrap_or(&format!("{}", default_config.peer_port))
                                              .trim().parse().map_err(|_e| "Invalid bitcoin:p2p_port value")?;

               if peer_port <= 1024 || peer_port >= 65535 {
                   return Err("Invalid p2p_port");
               }

               let rpc_port = bitcoin_section.get("port")
                                             .unwrap_or(&format!("{}", default_config.rpc_port))
                                             .trim().parse().map_err(|_e| "Invalid bitcoin:port value")?;

               if rpc_port <= 1024 || rpc_port >= 65535 {
                   return Err("Invalid rpc_port");
               }

               let username = bitcoin_section.get("user")
                                             .unwrap_or(&default_config.username);

               let password = bitcoin_section.get("password")
                                             .unwrap_or(&default_config.password);

               let timeout = bitcoin_section.get("timeout")
                                            .unwrap_or(&format!("{}", default_config.timeout))
                                            .trim().parse().map_err(|_e| "Invalid bitcoin:timeout value")?;

               let spv_headers_path = bitcoin_section.get("spv_headers_path")
                                            .unwrap_or(&default_config.spv_headers_path);

               let cfg = BitcoinIndexerConfig {
                   peer_host: peer_host.to_string(),
                   peer_port,
                   rpc_port,
                   username: username.to_string(),
                   password: password.to_string(),
                   timeout,
                   spv_headers_path: spv_headers_path.to_string()
               };
               return Ok(cfg);
           },
           Err(_) => {
               return Err("Failed to parse BitcoinConfigIndexer config file");
           }
       }
    }
}


impl BitcoinIndexerRuntime {
    pub fn default(network_id: u32) -> BitcoinIndexerRuntime {
        let mut rng = thread_rng();
        return BitcoinIndexerRuntime {
            sock: Arc::new(Mutex::new(None)),
            services: 0,
            user_agent: USER_AGENT.to_owned(),
            version_nonce: rng.gen(),
            magic: network_id
        };
    }
}


impl BitcoinIndexer {
    pub fn new() -> BitcoinIndexer {
        let default_config = BitcoinIndexerConfig::default();
        return BitcoinIndexer {
            config: default_config,
            runtime: BitcoinIndexerRuntime::default(BITCOIN_MAINNET)
        };
    }

    /// (re)connect to our configured network peer.
    /// Sets self.runtime.sock to a new socket referring to our configured
    /// Bitcoin peer.  If we fail to connect, this method sets the socket
    /// to None.
    fn reconnect_peer(&mut self) -> Result<(), &'static str> {
        match net::TcpStream::connect((self.config.peer_host.as_str(), self.config.peer_port)) {
            Ok(s) => {
                self.runtime.sock = Arc::new(Mutex::new(Some(s)));
                return Ok(());
            },
            Err(_e) => {
                self.runtime.sock = Arc::new(Mutex::new(None));
                return Err("Failed to connect to remote peer");
            }
        }
    }

    /// Get a locked handle to the internal socket 
    pub fn socket_locked(&mut self) -> LockResult<MutexGuard<Option<net::TcpStream>>> {
        return self.runtime.sock.lock();
    }
}


impl BurnchainIndexer for BitcoinIndexer {
    /// Instantiate the Bitcoin indexer, but don't connect to the peer network.
    /// Instead, load our configuration state and sanity-check it.
    /// Call connect() next.
    /// 
    /// Pass a directory (working_dir) that contains a "bitcoin.ini" file.
    fn setup(&mut self, working_dir: &str) -> Result<(), &'static str> {
       let mut conf_path = PathBuf::from(working_dir);
       conf_path.push("bitcoin.ini");

       match BitcoinIndexerConfig::from_file(conf_path.to_str().unwrap()) {
           Ok(cfg) => {
               self.config = cfg;
               return Ok(());
           },
           Err(e) => {
               return Err(e);
           }
       };
    }

    /// Connect to the Bitcoin peer network.
    /// Use the peer host and peer port given in the config file,
    /// and loaded in on setup.  Don't call this before setup().
    ///
    /// Pass "mainnet", "testnet", or "regtest" as the network name
    fn connect(&mut self, network_name: &str) -> Result<(), &'static str> {
        let network_id_opt = match network_name.as_ref() {
            "mainnet" => Some(BITCOIN_MAINNET),
            "testnet" => Some(BITCOIN_TESTNET),
            "regtest" => Some(BITCOIN_REGTEST),
            _ => None
        };

        if None == network_id_opt {
            return Err("Unrecognized network name");
        }

        let network_id = network_id_opt.unwrap();
        self.runtime = BitcoinIndexerRuntime::default(network_id);
        return self.reconnect_peer();
    }

    fn get_block_hash(&mut self, block_height: u64) -> Result<String, &'static str> {
        return Err("not implemented");
    }

    fn get_block_txs(&mut self, block_hash: &str) -> Result<Box<Vec<BurnchainTransaction>>, &'static str> {
        return Err("not implemented");
    }
}

