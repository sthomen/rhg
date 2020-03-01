mod revlog;

use std::io::Result;
use std::fs::File;
use revlog::Revlog;

fn main() -> Result<()> {
	let mut file = File::open("test.1")?;
	let revlog = Revlog::from_file(file);

	Ok(())
}
