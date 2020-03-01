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
 *
 * Total: 76 bytes
 */


#[derive(Debug)]
pub struct IndexV0 {
	offset: u32,
	length: u32,
	baserev: u32,
	linkrev: u32,
	parent1: [u8; 20],
	parent2: [u8; 20],
	nodeid: [u8;20]
}

impl IndexV0 {
	fn from_file(mut file: &File) -> Result<IndexV0> {
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
	parent1: u32,
	parent2: u32,
	nodeid: [u8; 32]
}

impl IndexNG {
	fn from_file(mut file: &File) -> Result<IndexNG> {
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

// shared with v1 and v2
pub const REVLOG_FLAG_INLINE_DATA: u32 = 1 << 16;

// This is only used by v1, it is implied in v2
pub const REVLOG_FLAG_GENERALDELTA: u32 = 1 << 17;

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
		let (version, flags) = Revlog::read_version(&file).unwrap();

		Revlog {
			version: version,
			flags: flags,
			index: vec![Revlog::read_index(&file, version).unwrap()]
		}
	}

	fn read_version(mut file: &File) -> Result<(u32, u32)> {
		let mut bytes = [0u8; 4];

		file.read_exact(&mut bytes)?;

		let number = BigEndian::read_u32(&bytes);
		let version = number & REVLOG_FLAG_MASK;
		let flags = number & !version;

		// rewind, so that we can read the first index normally
		file.seek(SeekFrom::Start(0))?;

		Ok((version, flags))
	}

	pub fn flag(&self, flag: u32) -> bool {
		self.flags & flag != 0
	}

	fn read_index(mut file: &File, version: u32) -> Result<Index> {
		let index: Index;

		match version {
			0 => index = Index::V0(IndexV0::from_file(file)?),
			1 => index = Index::NG(IndexNG::from_file(file)?),
			_ => panic!("Unknown file version!")
		}

		Ok(index)
	}
}
