use clap::{App, AppSettings, Arg};
use memmap::MmapOptions;
use std::env;
use std::fs::{self, File};
use std::path::PathBuf;

use anyhow::{anyhow, bail, ensure, Context, Result};

mod blueprint;
mod modinfo;

use modinfo::ModuleInfo;

#[derive(Debug)]
struct Arguments {
    module_info_path: PathBuf,
    android_top_path: PathBuf,
    command: Command,
}

#[derive(Debug)]
enum Command {
    List,
    Show(String, Option<String>),
    Source(String),
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
        .arg(
            Arg::with_name("android-top")
                .help("Path to top of Android tree")
                .long_help("Path to the top of the Android tree; defaults to `$ANDROID_BUILD_TOP`.")
                .long("android-top")
                .value_name("DIR")
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
        .subcommand(
            App::new("source")
                .about("Prints the Android.bp definition of module")
                .arg(Arg::with_name("NAME")
                    .help("Name of module to show")
                    .required(true))
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

    let android_top_path = if matches.is_present("android-top") {
        matches.value_of("android-top").unwrap().into()
    } else {
        env::var("ANDROID_BUILD_TOP")
            .map_err(|_| anyhow!("ANDROID_BUILD_TOP not set"))?
            .into()
    };

    let command = match &matches.subcommand() {
        ("list", _) => Command::List,
        ("show", Some(args)) => Command::Show(
            args.value_of("NAME")
                .expect("value guaranteed by clap")
                .to_string(),
            args.value_of("FIELD").map(|s| s.to_string()),
        ),
        ("source", Some(args)) => Command::Source(
            args.value_of("NAME")
                .expect("value guaranteed by clap")
                .to_string(),
        ),
        (_, _) => unreachable!(),
    };

    Ok(Arguments {
        module_info_path,
        android_top_path,
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
        Command::Source(name) => {
            let module = modinfo
                .find(&name)
                .ok_or_else(|| anyhow!("{}: module not found", name))??;
            ensure!(
                module.path.len() == 1,
                "{}: module does not have exactly one path: {:?}",
                name,
                module.path
            );
            let blueprint_path = format!(
                "{}/{}/Android.bp",
                args.android_top_path.display(),
                module.path[0]
            );
            let blueprint_contents = fs::read_to_string(&blueprint_path)
                .with_context(|| format!("could not read file {}", blueprint_path))?;
            let module_source = blueprint::find_module_source(&blueprint_contents, &name)?
                .ok_or_else(|| anyhow!("{}: module source not found", name))?;
            println!("{}", module_source);
        }
    }

    Ok(())
}
