extern crate byteorder;
extern crate hex;

use std::fs::File;
use std::string::String;
use std::io::{Result, Read, Seek, SeekFrom};
use byteorder::{ByteOrder, BigEndian};

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
 *
 * Total: 76 bytes
 */


#[derive(Debug)]
pub struct IndexV0 {
	offset: u32,
	length: u32,
	baserev: u32,
	linkrev: u32,
	pub parent1: [u8; 20],
	pub parent2: [u8; 20],
	nodeid: [u8; 20]
}

impl IndexV0 {
	pub fn from_file(mut file: &File) -> Result<IndexV0> {
		let mut bytes = [0u8; 16];
		let mut parent1 = [0u8; 20];
		let mut parent2 = [0u8; 20];
		let mut nodeid = [0u8; 20];

		file.read_exact(&mut bytes)?;
		file.read_exact(&mut parent1)?;
		file.read_exact(&mut parent2)?;
		file.read_exact(&mut nodeid)?;

		Ok(IndexV0 {
			offset: BigEndian::read_u32(&bytes[0..4]),
			length: BigEndian::read_u32(&bytes[4..8]),
			baserev: BigEndian::read_u32(&bytes[8..12]),
			linkrev: BigEndian::read_u32(&bytes[12..16]),
			parent1: parent1,
			parent2: parent2,
			nodeid: nodeid
		})
	}
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
 *
 * Total: 64 bytes
 */


#[derive(Debug)]
pub struct IndexNG {
	offset: u64,	// actually u48, but that doesn't exist
	flags: u16,
	length_compressed: u32,
	length: u32,
	baserev: u32,
	linkrev: u32,
	pub parent1: u32,
	pub parent2: u32,
	nodeid: [u8; 32]
}

impl IndexNG {
	pub fn from_file(mut file: &File) -> Result<IndexNG> {
		let mut bytes = [0u8; 32];
		let mut nodeid = [0u8; 32];

		let pos = file.seek(SeekFrom::Current(0))?;

		file.read_exact(&mut bytes)?;
		file.read_exact(&mut nodeid)?;

		Ok(IndexNG {
			// if we're at the start of the file, then the offset is the file
			// verison, not an actual offset
			offset:   if pos == 0 { 0 } else { BigEndian::read_u48(&bytes[0..6]) },
			flags:    BigEndian::read_u16(&bytes[6..8]),
			length_compressed: BigEndian::read_u32(&bytes[8..12]),
			length:   BigEndian::read_u32(&bytes[12..16]),
			baserev:  BigEndian::read_u32(&bytes[16..20]),
			linkrev:  BigEndian::read_u32(&bytes[20..24]),
			parent1:  BigEndian::read_u32(&bytes[24..28]),
			parent2:  BigEndian::read_u32(&bytes[28..32]),
			nodeid:   nodeid
		})
	}
}

#[derive(Debug)]
pub enum Index {
	V0(IndexV0),
	NG(IndexNG)
}

macro_rules! simple_disambiguation {
	($variant:ident, $parameter:ident) => {
		match *$variant {
			Index::V0(ref e) => e.$parameter,
			Index::NG(ref e) => e.$parameter
		}
	}
}

#[allow(dead_code)]
impl Index {
	/**
	 * Returns the offset of the data as an u64, even though the internal
	 * representation is eithe 32 bit or 48 bit
	 */
	pub fn offset(&self) -> u64 {
		match *self {
			Index::V0(ref e) => e.offset as u64,
			Index::NG(ref e) => e.offset
		}
	}

	/**
	 * Return the binary flags for this index, since V0 does not have flags,
	 * we return all zeroes.
	 */
	pub fn flags(&self) -> u16 {
		match *self {
			Index::NG(ref e) => e.flags,
			_ => 0
		}
	}

	/**
	 * Returns the length of the (compressed for NG) data entry.
	 */
	pub fn length(&self) -> u32 {
		match *self {
			Index::V0(ref e) => e.length,
			Index::NG(ref e) => e.length_compressed
		}
	}

	/**
	 * Base revision
	 */
	pub fn baserev(&self) -> u32 {
		simple_disambiguation!(self, baserev)
	}

	/**
	 * Link revision
	 */
	pub fn linkrev(&self) -> u32 {
		simple_disambiguation!(self, linkrev)
	}

	/**
	 * Node ID:s are of variable length between the versions, so this
	 * normalizes to 32 bytes and keeps the top bytes zero.
	 */
	pub fn nodeid(&self) -> [u8; 32] {
		match *self {
			Index::V0(ref e) => {
				let mut nodeid = [0u8;32];

				for n in 0 .. 20 {
					nodeid[n] = e.nodeid[n];
				}

				nodeid
			},
			Index::NG(ref e) => e.nodeid
		}
	}

	/**
	 * Return the node id as a hex-encoded string
	 */
	pub fn id(&self) -> String {
		hex::encode(&self.nodeid())
	}

	/**
	 * Return the short (six bytes) node id as a hex-encoded string
	 */
	pub fn short_id(&self) -> String {
		hex::encode(&self.nodeid()[0..6])
	}
}
