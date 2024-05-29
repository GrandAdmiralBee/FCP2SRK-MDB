use clap::{value_parser, Arg, ArgAction, Command};

pub fn cli() -> Option<Args> {
    let cli = Command::new("SRK-parser")
        .about("Parsing old FCP errors from .mdb files")
        .arg_required_else_help(true)
        .args([
            Arg::new("mdb-files")
                .short('M')
                .required(true)
                .long("path to .mdb files")
                .value_parser(value_parser!(String))
                .help("path to .mdb files")
                .action(ArgAction::Set)
                .num_args(1..),
            Arg::new("cpp-files")
                .short('C')
                .required(true)
                .long("path to .cpp files")
                .value_parser(value_parser!(String))
                .help("path to .cpp files")
                .action(ArgAction::Set)
                .num_args(1..),
        ]);

    let matches = cli.get_matches();

    let mdb_files: Vec<String> = matches
        .get_many("mdb-files")
        .expect("Expected paths to .mdb files")
        .cloned()
        .collect();
    let cpp_files: Vec<String> = matches
        .get_many("cpp-files")
        .expect("Expected paths to .cpp files")
        .cloned()
        .collect();

    for file in &mdb_files {
        if !std::path::Path::new(&file).exists() {
            println!("Path does not exist: {}", &file);
            return None;
        }
    }
    for file in &cpp_files {
        if !std::path::Path::new(&file).exists() {
            println!("Path does not exist: {}", &file);
            return None;
        }
    }

    Some(Args {
        mdb_files,
        cpp_files,
    })
}

pub struct Args {
    pub mdb_files: Vec<String>,
    pub cpp_files: Vec<String>,
}
