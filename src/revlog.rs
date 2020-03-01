extern crate byteorder;

use std::fs::File;
use std::io::{Result, Read, Seek, SeekFrom};
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

pub struct IndexV0 {
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

pub struct IndexNG {
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

pub enum Index {
	IndexV0,
	IndexNG
}

// shared with v1 and v2
const REVLOG_FLAG_INLINE_DATA: u32 = 1 << 16;

// This is only used by v1, it is implied in v2
const REVLOG_FLAG_GENERALDELTA: u32 = 1 << 17;

// XXX These are the two flags we know about, so let's generate a mask
// for the version number from those. Is there a general rule for which
// bits are reserved for flags? The python source and what documentation
// I've found does not tell.
const REVLOG_FLAG_MASK: u32 = !(REVLOG_FLAG_INLINE_DATA | REVLOG_FLAG_GENERALDELTA);

pub struct Revlog {
	pub version: u32,
	pub flags: u32,
	pub index: Vec<Index>
}

impl Revlog {
	pub fn from_file(file: File) -> Revlog {
		let (version, flags) = Revlog::read_version(file).unwrap();

		Revlog {
			version: version,
			flags: flags,
			index: vec![] // TODO load the actual index
		}
	}

	fn read_version(mut file: File) -> Result<(u32, u32)> {
		let mut bits = [0u8; 4];

		file.read_exact(&mut bits)?;

		let number = BigEndian::read_u32(&bits);
		let version = number & REVLOG_FLAG_MASK;
		let flags = number & !version;

		// rewind, so that we can read the first index normally
		file.seek(SeekFrom::Start(0))?;

		Ok((version, flags))
	}
}
