mod revlog;

use std::env;
use std::io::Result;

use revlog::Revlog;

fn main() -> Result<()> {
	let args: Vec<String> = env::args().collect();

	if args.len() > 1 {
		let revlog = Revlog::load(&args[1])?;

		println!("Revlog {}, version = {}, flags = 0x{:x}", revlog.filename, revlog.version, revlog.flags);

		for (index,entry) in revlog.index.iter().enumerate() {
			println!("Entry {}: {}, offset: {}, length: {}", entry.linkrev(), entry.short_id(), entry.offset(), entry.length());

			println!("{}", revlog.read_changeset(index as u64)?);
		}

		println!("{}", revlog.filename);
	}
	Ok(())
}
