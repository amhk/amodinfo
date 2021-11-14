use std::error::Error;
use std::fs;

mod error;
mod modinfo;

use modinfo::ModuleInfo;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args();
    args.next();
    if let Some(path) = args.next() {
        let data = fs::read_to_string(path)?;
        let modinfo = ModuleInfo::try_from(&*data)?;
        println!("{}", modinfo.module_names().join("\n"));
    }
    Ok(())
}
