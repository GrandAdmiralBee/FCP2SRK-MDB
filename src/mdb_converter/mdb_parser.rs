use rayon::prelude::*;
use regex::Regex;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use super::FCP;
use crate::tui::AppEvent;

pub fn get_loggers(
    paths: &[String],
    tx: std::sync::mpsc::Sender<AppEvent>,
) -> HashMap<FCP, HashMap<String, String>> {
    let mut loggers = HashMap::new();

    for mdb in paths {
        let msg = format!("Loading file {}", mdb);
        tx.send(AppEvent::Log(msg, crate::tui::log_list::LogLevel::Info))
            .unwrap();
        let file_name = std::path::Path::new(&mdb)
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap();

        let fcp = FCP::from_str(file_name).unwrap();
        let mdb_file = std::fs::read_to_string(&mdb).unwrap();
        let codes = get_mdb_codes(&mdb_file);
        loggers.insert(fcp, codes);
    }
    return loggers;
}

fn get_mdb_codes(mdb: &str) -> HashMap<String, String> {
    let re = Regex::new(r#"(?P<Code>\w+)"#).unwrap();

    let codes = Arc::new(RwLock::new(HashMap::new()));

    mdb.par_lines().for_each(|line| {
        let cap = re.captures(line);
        if cap.is_some() {
            let cap = cap.unwrap();
            let mdb = get_mdb(line);
            let code = &cap["Code"];

            let codes = Arc::clone(&codes);
            let mut codes = codes.write().expect("RwLock Poisoned");
            codes.insert(code.to_string(), mdb);
        }
    });

    let codes = codes.read().unwrap().clone();
    codes
}

fn get_mdb(mdb: &str) -> String {
    let re = Regex::new(r#"\w+\s+(?P<mdb>\".+\")"#).unwrap();
    let cap = re.captures(mdb).unwrap();
    let mut mdb_result = cap["mdb"].to_string();

    mdb_result
}
