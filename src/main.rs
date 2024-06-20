use clap::{App, AppSettings, Arg};
use memmap::MmapOptions;
use std::env;
use std::fs::File;
use std::path::PathBuf;

use anyhow::{anyhow, bail, Result};

mod blueprint;
mod modinfo;

use modinfo::ModuleInfo;

#[derive(Debug)]
struct Arguments {
    module_info_path: PathBuf,
    command: Command,
}

#[derive(Debug)]
enum Command {
    List,
    Show(String, Option<String>),
}

const MODULE_FIELDS: [&str; 9] = [
    "name",
    "path",
    "installed",
    "dependencies",
    "class",
    "supported_variants",
    "shared_libs",
    "static_libs",
    "system_shared_libs",
];

fn parse_args() -> Result<Arguments> {
    let matches = App::new("amodinfo")
        .version(env!("CARGO_PKG_VERSION"))
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg(
            Arg::with_name("module-info")
                .help("Path to module-info.json")
                .long_help("Path to module-info.json; defaults to `$ANDROID_PRODUCT_OUT/module-info.json` if $ANDROID_PRODUCT_OUT is set.")
                .long("module-info")
                .value_name("FILE")
                .takes_value(true),
        )
        .subcommand(App::new("list").about("Prints the names of all modules"))
        .subcommand(
            App::new("show")
                .about("Prints information about a given module")
                .arg(Arg::with_name("NAME")
                    .help("Name of module to show")
                    .required(true))
                .arg(Arg::with_name("FIELD")
                    .help("Name of field to show")
                    .possible_values(&MODULE_FIELDS)),
        )
        .get_matches();

    let module_info_path = if matches.is_present("module-info") {
        matches.value_of("module-info").unwrap().into()
    } else {
        let prefix =
            env::var("ANDROID_PRODUCT_OUT").map_err(|_| anyhow!("ANDROID_PRODUCT_OUT not set"))?;
        let mut path = PathBuf::from(prefix);
        path.push("module-info.json");
        path
    };

    let command = match &matches.subcommand() {
        ("list", _) => Command::List,
        ("show", Some(args)) => Command::Show(
            args.value_of("NAME")
                .expect("value guaranteed by clap")
                .to_string(),
            args.value_of("FIELD").map(|s| s.to_string()),
        ),
        (_, _) => unreachable!(),
    };

    Ok(Arguments {
        module_info_path,
        command,
    })
}

fn print_field(field: Option<Vec<&str>>) {
    if let Some(vec) = field {
        println!("{}", vec.join("\n"));
    }
}

fn main() -> Result<()> {
    let args = parse_args()?;

    let file = File::open(args.module_info_path)?;
    // SAFETY: assume the underlying file will not be mutated while this program is running,
    // thus making it safe to memory map the file
    let mmap = unsafe { MmapOptions::new().map(&file)? };
    let data = std::str::from_utf8(&mmap)?;
    let modinfo = ModuleInfo::try_from(data)?;

    match args.command {
        Command::List => {
            println!("{}", modinfo.module_names().join("\n"));
        }
        Command::Show(name, field) => {
            let module = modinfo
                .find(&name)
                .ok_or_else(|| anyhow!("{}: module not found", name))??;

            if let Some(f) = &field {
                match f.as_str() {
                    "name" => println!("{}", module.name),
                    "path" => print_field(Some(module.path)),
                    "installed" => print_field(module.installed),
                    "dependencies" => print_field(module.dependencies),
                    "class" => print_field(Some(module.class)),
                    "supported_variants" => print_field(module.supported_variants),
                    "shared_libs" => print_field(module.shared_libs),
                    "static_libs" => print_field(module.static_libs),
                    "system_shared_libs" => print_field(module.system_shared_libs),
                    _ => {
                        bail!("{}: unknown field", field.unwrap());
                    }
                }
            } else {
                println!("{:#?}", module);
            }
        }
    }

    Ok(())
}
