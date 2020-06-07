extern crate flate2;
extern crate datetime;

use std::fmt;
use std::io::Read;
use std::string::String;
use datetime::{OffsetDateTime,LocalDateTime,Offset};
use flate2::bufread::ZlibDecoder;

#[derive(Debug)]
pub struct Changeset {
	pub data: Vec<u8>,
	pub files: Vec<String>,
	pub hash: Option<String>,
	pub author: Option<String>,
	pub when: Option<OffsetDateTime>,
	pub message: Option<String>
}



impl Changeset {
	pub fn from(data: Vec<u8>) -> Changeset {
		let mut changeset = Changeset {
			data: data.to_vec(),
			files: Vec::new(),
			hash: None,
			author: None,
			when: None,
			message: None
		};

		changeset.decode();

		return changeset
	}

	fn raw(&self) -> String {
		match self.data[0] {
			// '\0', an empty changeset, XXX is this how it works?
			0 => String::new(),

			// 'x' zlib compressed, this is the most common
			120 => {
				let mut zlib = ZlibDecoder::new(&self.data[..]);
				let mut s = String::new();
				zlib.read_to_string(&mut s);
				s
			},

			// 'u', unicode data? The python code just returns a
			// buffer object with an offset of 1, meaning the
			// code prefix is removed.
			117 => String::from_utf8(self.data[1..].to_vec()).unwrap(),

			// Changesets without indicator should be attempted to
			// be uncompressed with the discovered available
			// compressors.
			//
			// The python code records the compression algorithm
			// used for a given key, and then re-uses that every
			// time it discovers it (this would be an efficiency
			// hack in order to not have to reinitialize the
			// decompressor every time)
			_  => panic!("Unknown compression format! Arbitrary compression formats are not implemented. Fix me.")
		}
	}

	fn decode(&mut self) {
		let uncompressed = self.raw();
		let mut header: bool = true;
		let mut message = Vec::new();

		for (index,line) in uncompressed.lines().enumerate() {
			if line == "" {
				header = false;
				continue;
			}

			if header {
				match index {
					0 => self.hash = Some(String::from(line)),
					1 => self.author = Some(String::from(line)),
					2 => self.when = Some(Changeset::parse_date_string(&line)),
					_ => self.files.push(String::from(line))
				}
			} else {
				message.push(line);
			}
		}

		self.message = Some(message.join("\n"));
	}

	fn parse_date_string(string: &str) -> OffsetDateTime {
		let mut bits = string.split(" ");

		let timestamp = bits.next().unwrap().parse::<i64>().unwrap();
		let utcoffset = bits.next().unwrap().parse::<i32>().unwrap();

		OffsetDateTime {
			local: LocalDateTime::at(timestamp),
			offset: Offset::of_seconds(utcoffset).unwrap()
		}
	}
}

impl fmt::Display for Changeset {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "Author: {}\nDate: XXX\nFiles: {}\n\n{}\n",
			self.author.as_ref().unwrap(),
//			self.when.unwrap(), OffsetDateTime is being a bitch
			self.files.join(", "),
			self.message.as_ref().unwrap())
	}
}
