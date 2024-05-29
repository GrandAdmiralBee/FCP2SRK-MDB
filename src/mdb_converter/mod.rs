pub mod mdb_parser;
pub mod parser;

use colored::*;

use std::collections::HashMap;

pub fn start_converter() {
    let cli = crate::cli::cli().unwrap();
    let mdb_files = cli.mdb_files;
    let cpp_files = cli.cpp_files;

    //let mdb_files = vec!["mdb/fcpasm.mdb".to_string(), "mdb/fcpse.mdb".to_string()];
    //let cpp_files = vec!["cpp/FcpAsm.cpp".to_string()];

    let loggers = mdb_parser::get_loggers(&mdb_files);

    for file_name in cpp_files {
        let str = format!("------------ Editing {file_name} -------------").green();
        println!("{str}");

        let mut logger_map = HashMap::new();

        loggers.iter().for_each(|(logger, _)| {
            println!(
                "Specify variable name for logger {:?} (for auto-search based on comments)",
                logger
            );
            let mut name = String::new();
            std::io::stdin().read_line(&mut name).unwrap();
            let name = name.trim().to_string();
            logger_map.insert(name, *logger);
        });

        let buffer = std::fs::read_to_string(&file_name).unwrap();
        let str = format!("-------------- Removing multi-line logs --------------").green();
        println!("{str}");
        let buffer = parser::join_log_lines(&buffer);
        let comments = parser::find_comment_around_line(&buffer, 575);
        dbg!(comments);
        let str = format!("-------------- Replacing logs --------------").green();
        println!("{str}");

        let mut res = String::new();
        let mut line_num = 0;
        for line in buffer.lines() {
            line_num += 1;
            if !line.trim().starts_with("static")
                && !line.trim().starts_with("static")
                && !line.trim().starts_with("#")
            {
                res.push('\n');
                res.push_str("static QString mdb_message;");
            }
            if line.trim().is_empty() {
                continue;
            }
            let new_line = parser::parse_line(line, &loggers, &logger_map, &buffer, line_num);
            if new_line.is_none() {
                if res.len() > 0 {
                    res.push('\n');
                }
                res.push_str(line);
            } else {
                if res.len() > 0 {
                    res.push('\n');
                }
                res.push_str(&new_line.unwrap());
            }
        }

        if !std::path::Path::new("output").exists() {
            std::fs::create_dir("output").expect("Couldn't create output directory!");
        }
        let path = std::path::Path::new(&file_name);
        let file_stem = path
            .file_stem()
            .expect(&format!("Couldn't take file stem for file {}", file_name))
            .to_str()
            .expect("Path contains non-valid UTF-8");
        std::fs::write(&format!("output/{}.out", file_stem), res)
            .expect("Couldn't write output file");
    }
}

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
enum FCP {
    SE,
    ASM,
    DP,
    DRC,
    EDIF,
    ERP,
    GDSREADER,
    IO,
    LMAN,
    ME,
    REPORTS,
    SDB,
    SHELL,
    TDM,
    UI,
}

impl FCP {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "fcpasm" => Some(Self::ASM),
            "fcpdp" => Some(Self::DP),
            "fcpdrc" => Some(Self::DRC),
            "fcpedif" => Some(Self::EDIF),
            "fcperp" => Some(Self::ERP),
            "fcpgdsreader" => Some(Self::GDSREADER),
            "fcpio" => Some(Self::IO),
            "fcplman" => Some(Self::LMAN),
            "fcpme" => Some(Self::ME),
            "fcpreports" => Some(Self::REPORTS),
            "fcpsdb" => Some(Self::SDB),
            "fcpse" => Some(Self::SE),
            "fcpshell" => Some(Self::SHELL),
            "fcptdm" => Some(Self::TDM),
            "fcpui" => Some(Self::UI),
            _ => None,
        }
    }

    pub fn to_str(&self) -> String {
        let s = match self {
            Self::ASM => "fcpasm",
            Self::DP => "fcpdp",
            Self::DRC => "fcpdrc",
            Self::EDIF => "fcpedif",
            Self::ERP => "fcperp",
            Self::GDSREADER => "fcpgdsreader",
            Self::IO => "fcpio",
            Self::LMAN => "fcplman",
            Self::ME => "fcpme",
            Self::REPORTS => "fcpreports",
            Self::SDB => "fcpsdb",
            Self::SE => "fcpse",
            Self::SHELL => "fcpshell",
            Self::TDM => "fcptdm",
            Self::UI => "fcpui",
        };
        s.to_string()
    }
}
