use clap::{App, AppSettings, Arg};
use memmap::MmapOptions;
use std::env;
use std::error::Error;
use std::fs::File;
use std::path::PathBuf;

mod error;
mod modinfo;

use error::CLIError;
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

fn parse_args() -> Result<Arguments, CLIError> {
    let matches = App::new("amodinfo")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg(
            Arg::with_name("module-info")
                .long("module-info")
                .value_name("FILE")
                .takes_value(true),
        )
        .subcommand(App::new("list"))
        .subcommand(
            App::new("show")
                .arg(Arg::with_name("NAME").required(true))
                .arg(Arg::with_name("FIELD")),
        )
        .get_matches();

    let module_info_path = if matches.is_present("module-info") {
        matches.value_of("module-info").unwrap().into()
    } else {
        let prefix = env::var("ANDROID_PRODUCT_OUT")
            .map_err(|_| CLIError("ANDROID_PRODUCT_OUT not set".to_string()))?;
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

fn main() -> Result<(), Box<dyn Error>> {
    let args = parse_args()?;

    let file = File::open(args.module_info_path)?;
    // SAFETY: assume the underlying file will not be mutated while this program is running,
    // thus making it safe to memory map the file
    let mmap = unsafe { MmapOptions::new().map(&file)? };
    let data = std::str::from_utf8(&mmap)?;
    let modinfo = ModuleInfo::try_from(&*data)?;

    match args.command {
        Command::List => {
            println!("{}", modinfo.module_names().join("\n"));
        }
        Command::Show(name, field) => {
            let module = modinfo
                .find(&name)
                .ok_or_else(|| CLIError(format!("{}: module not found", name)))??;

            if let Some(f) = &field {
                match f.as_str() {
                    "name" => println!("{}", module.name),
                    "path" => println!("{}", module.path.join("\n")),
                    "installed" => println!("{}", module.installed.join("\n")),
                    "dependencies" => println!("{}", module.dependencies.join("\n")),
                    "class" => println!("{}", module.class.join("\n")),
                    "tags" => println!("{}", module.tags.join("\n")),
                    "test_config" => println!("{}", module.test_config.join("\n")),
                    _ => {
                        return Err(Box::new(CLIError(format!(
                            "{}: unknown field",
                            field.unwrap()
                        ))))
                    }
                }
            } else {
                println!("{:#?}", module);
            }
        }
    }

    Ok(())
}
