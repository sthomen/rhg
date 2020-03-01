extern crate byteorder;

use std::fs::File;
use std::io::{Result, Read};
use byteorder::{ByteOrder, BigEndian};

/*
 * Mercurial stores integers bigendian
 *
 * The first 4 bytes in the index file indicates the version number since it
 * would always be 0, in the next block, it's the offset.
 */

/*
 * index v0:
 *
 *  4 bytes: offset
 *  4 bytes: compressed length
 *  4 bytes: base rev
 *  4 bytes: link rev
 * 20 bytes: parent 1 nodeid
 * 20 bytes: parent 2 nodeid
 * 20 bytes: nodeid
 */

struct IndexV0 {
	offset: u32,
	length: u32,
	baserev: u32,
	linkrev: u32,
	parent1: [u8; 20],
	parent2: [u8; 20],
	nodeid: [u8;20]
}

/*
 * index NG
 *
 *  6 bytes: offset
 *  2 bytes: flags
 *  4 bytes: compressed length
 *  4 bytes: uncompressed length
 *  4 bytes: base rev
 *  4 bytes: link rev
 *  4 bytes: parent 1 rev
 *  4 bytes: parent 2 rev
 * 32 bytes: nodeid
 */

struct IndexNG {
	offset: u64,
	flags: u16,
	length_compressed: u32,
	length: u32,
	baserev: u32,
	linkrev: u32,
	parent1: u32,
	parent2: u32,
	nodeid: [u8; 32]
}

enum Index {
	IndexV0,
	IndexNG
}

pub struct Revlog {
	index: Index
}

impl Revlog {
	pub fn from_file(mut file: File) {
		let version: u32 = Revlog::read_version_and_rewind(file);
		
		println!("{:x}", version);
	}

	fn read_version_and_rewind(mut file: File) -> u32 {
		let mut bits = [0u8; 4];
		file.read_exact(&mut bits);
		return BigEndian::read_u32(&bits);
	}
}
