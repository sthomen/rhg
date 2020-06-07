mod revlog;

use std::env;
use std::io::Result;
use std::fs::File;

use revlog::Revlog;

fn main() -> Result<()> {
	let args: Vec<String> = env::args().collect();

	if args.len() > 1 {
		let file = File::open(&args[1])?;
		let revlog = Revlog::from_file(&file)?;

		println!("Revlog version = {}, flags = 0x{:x}", revlog.version, revlog.flags);

		for (index,entry) in revlog.index.iter().enumerate() {
			println!("Entry {}: {}, offset: {}, length: {}", entry.linkrev(), entry.short_id(), entry.offset(), entry.length());

			println!("{}", revlog.read_changeset(&file, index as u64)?);
		}
	}
	Ok(())
}
