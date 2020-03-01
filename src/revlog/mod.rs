extern crate byteorder;

mod index;
use index::*;

use std::fs::File;
use std::io::{Result, Read, Seek, SeekFrom};
use byteorder::{ByteOrder, BigEndian};

/*
 * Mercurial stores integers bigendian
 *
 * The first 4 bytes in the index file indicates the version number since it
 * would always be 0, in the next block, it's the offset.
 */

// shared with v1 and v2
pub const REVLOG_FLAG_INLINE_DATA: u32 = 1 << 16;

// This is only used by v1, it is implied in v2
pub const REVLOG_FLAG_GENERALDELTA: u32 = 1 << 17;

// XXX These are the two flags we know about, so let's generate a mask
// for the version number from those. Is there a general rule for which
// bits are reserved for flags? The python source and what documentation
// I've found does not tell.
// Give then values above though, I'm gussing the higher 16 bits are the flags.
const REVLOG_FLAG_MASK: u32 = 0x0000ffff;

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

	fn read_index(file: &File, version: u32) -> Result<Index> {
		let index: Index;

		match version {
			0 => index = Index::V0(IndexV0::from_file(file)?),
			1 => index = Index::NG(IndexNG::from_file(file)?),
			_ => panic!("Unknown file version {}!", version)
		}

		Ok(index)
	}
}
