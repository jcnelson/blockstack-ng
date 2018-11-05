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

use std::fs;
use std::io;

use bitcoin::blockdata::block::{LoneBlockHeader, BlockHeader};
use bitcoin::network::encodable::{ConsensusEncodable, ConsensusDecodable};
use bitcoin::network::serialize::{RawEncoder, RawDecoder};
use bitcoin::util::hash::Sha256dHash;

use burnchains::bitcoin::indexer::BitcoinIndexer;
use burnchains::bitcoin::Error as net_error;

const BLOCK_HEADER_SIZE: u64 = 81;

/*
/// Report how many block headers we have downloaded to the given path.
/// Returns Ok(-1) if there are none, but the file otherwise exists
pub fn get_headers_height(headers_path: &str) -> Result<u64, net_error> {
    let metadata_result = fs::metadata(headers_path);
    if metadata_result.is_err() {
        return metadata_result;
    }

    let metadata = metadata_result.unwrap();
    let file_size = metadata.len();
    return Ok((file_size / BLOCK_HEADER_SIZE) - 1);
}

/// Read the block header at a particular height 
pub fn read_block_header(headers_path: &str, block_height: u64) -> Result<BlockHeader, net_error> {
    let headers_file = fs::File::open(headers_path)
                        .map_err(net_error::FilesystemError)?;

    headers_file.seek(fs::SeekFrom::start(BLOCK_HEADER_SIZE * block_height)
                        .map_err(net_error::FilesystemError)?;

}

/// Append block headers to our SPV headers file.
/// In doing so, make sure the headers are contiguous in the chain.
/// Does NOT verify that the difficulty was consistent.
pub fn append_block_headers(indexer: &mut BitcoinIndexer, headers: &mut vec<LoneBlockHeader>) -> Result<(), net_error> {
    let spv_headers_path = indexer.config.spv_headers_path;
    let spv_height = get_headers_height(indexer)?;

    let prev_block_hash = 
        if spv_height >= 0 {
            // adding headers -- find the last block hash 


    if spv_height < 0 {
        // first time we're getting headers

*/
