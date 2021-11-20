use memmap::MmapOptions;
use std::error::Error;
use std::fs::File;

mod error;
mod modinfo;

use modinfo::ModuleInfo;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args();
    args.next();
    if let Some(path) = args.next() {
        let file = File::open(path)?;
        // SAFETY: assume the underlying file will not be mutated while this program is running,
        // thus making it safe to memory map the file
        let mmap = unsafe { MmapOptions::new().map(&file)? };
        let data = std::str::from_utf8(&mmap)?;
        let modinfo = ModuleInfo::try_from(&*data)?;
        if let Some(name) = args.next() {
            println!("{:?}", modinfo.find(&name));
        } else {
            println!("{}", modinfo.module_names().join("\n"));
        }
    }
    Ok(())
}
