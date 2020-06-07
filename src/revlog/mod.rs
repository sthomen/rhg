#![allow(dead_code)]
// We're allowing dead code here because REVLOG_FLAG_GENERALDELTA is really
// only useful occasionally and for old versions of the revlog, so it would
// ALWAYS throw a damn warning, as well as has_flag() being mostly useful
// outside this crate.

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
	pub fn from_file(mut file: &File) -> Result<Revlog> {
		let (version, flags, length) = Revlog::read_version(&file).unwrap();
		let mut index = Vec::new();

		loop {
			let entry = Revlog::read_index(&file, version)?;

			// TODO instead of just skiping here, load the changeset data somewhere
			if flags & REVLOG_FLAG_INLINE_DATA != 0 {
				file.seek(SeekFrom::Current(entry.length() as i64))?;
			}

			index.push(entry);

			// At the exact end, loading is successful
			if file.seek(SeekFrom::Current(0))? == length {
				break;
			}
		}

		Ok(Revlog {
			version: version,
			flags: flags,
			index: index
		})
	}

	fn read_version(mut file: &File) -> Result<(u32, u32, u64)> {
		let mut bytes = [0u8; 4];

		// load file version info
		file.read_exact(&mut bytes)?;

		// ..parse
		let number = BigEndian::read_u32(&bytes);
		let version = number & REVLOG_FLAG_MASK;
		let flags = number & !version;

		// find the length of the file
		let length = file.seek(SeekFrom::End(0))?;

		// rewind, so that we can read the first index normally
		file.seek(SeekFrom::Start(0))?;

		Ok((version, flags, length))
	}

	pub fn has_flag(&self, flag: u32) -> bool {
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

	pub fn read_data(&self, mut file: &File, index: u64) -> Result<(Vec<u8>)> {
		// find the entry in our index
		let entry = &self.index[index as usize];

		// load its length
		let length = entry.length();
		let offset;

		// make some room to load the data
		let mut buffer = vec![0u8; length as usize];

		// calculate the actual offset
		if self.has_flag(REVLOG_FLAG_INLINE_DATA) {
			offset = entry.offset() + entry.size() * (index + 1);
		} else {
			offset = entry.offset();
		}

		// seek & load
		file.seek(SeekFrom::Start(offset))?;
		file.read_exact(&mut buffer)?;

		Ok(buffer)
	}
}
