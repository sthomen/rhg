mod revlog;

use std::env;
use std::io::Result;
use std::fs::File;
use revlog::Revlog;

fn main() -> Result<()> {
	let args: Vec<String> = env::args().collect();

	if args.len() > 2 {
		let file = File::open(&args[1])?;
		let revlog = Revlog::from_file(file);

		println!("Revlog version = {}, flags = 0x{:x}", revlog.version, revlog.flags);
	}
	Ok(())
}
