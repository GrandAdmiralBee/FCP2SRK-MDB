use colored::*;
use rayon::join;
use regex::Regex;

use std::collections::HashMap;

use super::FCP;
use crate::tui::{log_list::LogLevel, AppEvent};

pub fn parser(
    cli: crate::cli::Args,
    tx: std::sync::mpsc::Sender<AppEvent>,
    app2parser_receiver: std::sync::mpsc::Receiver<AppEvent>,
) {
    let mdb_files = cli.mdb_files;
    let cpp_files = cli.cpp_files;
    //let mdb_files = vec!["mdb/fcpasm.mdb".to_string(), "mdb/fcpse.mdb".to_string()];
    //let cpp_files = vec!["cpp/FcpAsm.cpp".to_string()];

    let loggers = super::mdb_parser::get_loggers(&mdb_files, tx.clone());

    for file_name in cpp_files {
        let str = format!("------------ Editing {file_name} -------------");
        tx.send(AppEvent::Log(str, LogLevel::Info)).unwrap();

        let mut logger_map = HashMap::new();

        loggers.iter().for_each(|(logger, _)| {
            let msg = format!(
                "Specify variable name for logger {:?} (for auto-search based on comments)",
                logger
            );
            tx.send(AppEvent::Log(msg, LogLevel::Info)).unwrap();
            tx.send(AppEvent::WaitForInput).unwrap();
            let mut name = String::new();
            for event in &app2parser_receiver {
                match event {
                    AppEvent::Command(n) => {
                        name = n.trim().to_string();
                        tx.send(AppEvent::Log(name.clone(), LogLevel::Info))
                            .unwrap();
                        break;
                    }
                    _ => (),
                }
            }
            logger_map.insert(name, *logger);
        });

        let buffer = std::fs::read_to_string(&file_name).unwrap();
        let str = format!("-------------- Removing multi-line logs --------------");
        tx.send(AppEvent::Log(str, LogLevel::Info)).unwrap();
        let buffer = join_log_lines(&buffer, tx.clone());
        tx.send(AppEvent::NewFile(buffer.clone())).unwrap();
        let str = format!("-------------- Replacing logs --------------");
        tx.send(AppEvent::Log(str, LogLevel::Info)).unwrap();
        tx.send(AppEvent::Log(
            "Specify line number to put static variable declaration".to_string(),
            LogLevel::Info,
        ))
        .unwrap();

        let mut static_line_num = 0;
        loop {
            tx.send(AppEvent::WaitForInput).unwrap();
            let mut name = String::new();
            for event in &app2parser_receiver {
                match event {
                    AppEvent::Command(n) => {
                        name = n.trim().to_string();
                        tx.send(AppEvent::Log(name.clone(), LogLevel::Info))
                            .unwrap();
                        break;
                    }
                    _ => (),
                }
            }
            if let Ok(num) = name.trim().parse::<usize>() {
                static_line_num = num;
                break;
            } else {
                tx.send(AppEvent::Log("Wrong input".to_string(), LogLevel::Error))
                    .unwrap();
            }
        }

        let mut res = String::new();
        let mut line_num = 0;
        let mut recording_line_num = 0;
        for line in buffer.lines() {
            line_num += 1;
            recording_line_num += 1;
            if line_num == static_line_num {
                res.push('\n');
                let str = "static QString mdb_message;";
                res.push_str(str);
                tx.send(AppEvent::InsertFileLine(line_num, str.to_string()))
                    .unwrap();
                recording_line_num += 1;
            }
            if line.trim().is_empty() {
                continue;
            }
            let new_line = parse_line(
                line,
                &loggers,
                &logger_map,
                &buffer,
                line_num,
                recording_line_num,
                tx.clone(),
                &app2parser_receiver,
            );
            if new_line.is_none() {
                if res.len() > 0 {
                    res.push('\n');
                }
                res.push_str(line);
            } else {
                if res.len() > 0 {
                    res.push('\n');
                }
                let new_line = new_line.unwrap();
                res.push_str(&new_line);
                tx.send(AppEvent::ReplaceFileLine(recording_line_num, new_line))
                    .unwrap();
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

        tx.send(AppEvent::ReadyToQuit).unwrap();
    }
}

pub fn join_log_lines(buffer: &String, tx: std::sync::mpsc::Sender<AppEvent>) -> String {
    let mut res = String::new();

    let mut split_lines = String::new();
    let mut joined = true;

    let mut curr_line_num = 0;
    let mut new_line_num = 0;

    for line in buffer.lines() {
        curr_line_num += 1;
        if !joined {
            split_lines.push_str(&format!(" {}", line.trim()));

            tx.send(AppEvent::Log(line.to_string(), LogLevel::Trace))
                .unwrap();

            if line.ends_with(";") {
                res.push_str(&split_lines);
                res.push('\n');
                let msg = format!(
                    "------ Above is replaced with (now on line {})------\n{}",
                    new_line_num, split_lines
                );
                tx.send(AppEvent::Log(msg.to_string(), LogLevel::Trace))
                    .unwrap();
                split_lines.clear();
                joined = true;
            }
        } else {
            new_line_num += 1;
            let re = Regex::new("(qCritical|qInfo|qWarning)").unwrap();
            let cap = re.captures(line);
            if cap.is_none() {
                res.push_str(line);
                res.push('\n');
                continue;
            }

            if !line.ends_with(";") {
                joined = false;
                split_lines.push_str(line);
                let msg = format!(
                    "------ Multiple-line logs starting on line {} (now on line {}) ------\n{}",
                    curr_line_num,
                    new_line_num,
                    line.yellow()
                );
                tx.send(AppEvent::Log(msg.to_string(), LogLevel::Info))
                    .unwrap();
                continue;
            }

            let msg = format!(
                "------ One-line log on line {} (now on line {}) ------\n{}",
                curr_line_num, new_line_num, line
            );
            tx.send(AppEvent::Log(msg.to_string(), LogLevel::Trace))
                .unwrap();

            res.push_str(line);
            res.push('\n');
        }
    }

    return res;
}

pub fn parse_line(
    line: &str,
    loggers: &HashMap<FCP, HashMap<String, String>>,
    logger_map: &HashMap<String, FCP>,
    file: &str,
    line_num: usize,
    recoeding_line_num: usize,
    tx: std::sync::mpsc::Sender<AppEvent>,
    app2parser_receiver: &std::sync::mpsc::Receiver<AppEvent>,
) -> Option<String> {
    let re = Regex::new("(qCritical|qInfo|qWarning)").unwrap();
    let cap = re.captures(line);
    if cap.is_none() {
        return None;
    }

    let re = Regex::new(r#"\"\s*(?P<Err>\w+)+\s*\"\s*(?P<Strings>(?:<<\s*\S+\s*)*);"#).unwrap();
    let cap = re.captures(line);
    let cap = match cap {
        None => return None,
        Some(_) => cap.unwrap(),
    };
    let err: &str = &cap["Err"];
    let strings = &cap["Strings"].to_string();
    let strings = strings.replace("<", "");

    let re = Regex::new(r#"(\S+)"#).unwrap();
    let mut strings_vec = vec![];
    for (_, [string]) in re.captures_iter(&strings).map(|c| c.extract()) {
        strings_vec.push(string);
    }

    let mut commented_lines = find_comment_around_line(file, line_num as usize);
    if !commented_lines.contains(err) {
        commented_lines.clear();
    }
    let mut mdb_match = String::new();

    if !commented_lines.is_empty() {
        let msg = format!(
            "--------- For line {line_num} -------\n{}\n--------- found comments\n{}",
            line.trim(),
            commented_lines.trim()
        );

        tx.send(AppEvent::Log(msg, LogLevel::Trace)).unwrap();

        logger_map.iter().for_each(|(k, fcp)| {
            if !k.is_empty() {
                if commented_lines.contains(k) {
                    let codes = loggers.get(fcp).unwrap();
                    let opt = codes.get(err);

                    if opt.is_none() {
                        let msg = format!("No error code for {err} in {k}");
                        tx.send(AppEvent::Log(msg, LogLevel::Error)).unwrap();
                    } else {
                        mdb_match = opt.unwrap().to_string();
                        let msg = format!("Got {} for {} code in {}", mdb_match, err, k);
                        tx.send(AppEvent::Log(msg, LogLevel::Info)).unwrap();
                    }
                }
            }
        })
    } else {
        let mut base_str = format!(
            "---------For line {line_num} select mdb file----------\n{}\n",
            line.trim()
        );

        let mut count = 0;
        let mut fcp_vec = vec![];
        logger_map.iter().for_each(|(_, fcp)| {
            count += 1;
            fcp_vec.push(*fcp);
            base_str.push_str(&format!("{} - {}\n", count, fcp.to_str()));
        });

        tx.send(AppEvent::Log(base_str, LogLevel::Info)).unwrap();
        tx.send(AppEvent::JumpLine(recoeding_line_num)).unwrap();

        loop {
            tx.send(AppEvent::WaitForInput).unwrap();
            let mut name = String::new();
            for event in app2parser_receiver {
                match event {
                    AppEvent::Command(n) => {
                        name = n.trim().to_string();
                        tx.send(AppEvent::Log(name.clone(), LogLevel::Info))
                            .unwrap();
                        break;
                    }
                    _ => (),
                }
            }

            match name.trim().parse::<usize>() {
                Ok(num) => {
                    if num > fcp_vec.len() {
                        let msg = format!("wrong input! try again");
                        tx.send(AppEvent::Log(msg, LogLevel::Error)).unwrap();
                        continue;
                    } else {
                        let fcp = fcp_vec[num - 1];
                        let codes = loggers.get(&fcp).unwrap();
                        let opt = codes.get(err);

                        if opt.is_none() {
                            let msg = format!("No error code for {err} in {}", fcp.to_str());
                            tx.send(AppEvent::Log(msg, LogLevel::Error)).unwrap();
                        } else {
                            mdb_match = opt.unwrap().to_string();
                            let msg =
                                format!("Got {} for {} code in {}", mdb_match, err, fcp.to_str());
                            tx.send(AppEvent::Log(msg, LogLevel::Info)).unwrap();
                            break;
                        }
                    }
                }
                Err(..) => {
                    let msg = format!("wrong input! try again");
                    tx.send(AppEvent::Log(msg, LogLevel::Error)).unwrap();
                }
            }
        }
    }

    let mut new_line = format!("mdb_message = QString({})", mdb_match);
    for string in strings_vec {
        new_line = format!("{}.arg({})", new_line, string);
    }
    new_line.push(';');

    if line.contains("qCritical") {
        new_line.push_str(" qCritical() << mdb_message;");
    } else if line.contains("qInfo") {
        new_line.push_str(" qInfo() << mdb_message;");
    } else if line.contains("qWarning") {
        new_line.push_str(" qWarning() << mdb_message;");
    };
    let re = Regex::new(r#"(?P<spaces>.*)(?:qCritical|qInfo|qWarning)"#).unwrap();
    let caps = re.captures(line).unwrap();
    let spaces = &caps["spaces"];
    new_line = format!("{}{}", spaces, new_line);

    let msg = format!(
        "------- Replacing log on line {} --------\n{}\n{}",
        line_num,
        line.yellow(),
        new_line.green()
    );

    tx.send(AppEvent::Log(msg, LogLevel::Trace)).unwrap();
    Some(new_line)
}

pub fn find_comment_around_line(file: &str, line_num: usize) -> String {
    let mut result = String::new();

    let mut curr_line = 0;
    for line in file.lines() {
        curr_line += 1;
        if curr_line <= line_num {
            if line.trim().starts_with("//") {
                if result.len() > 0 {
                    result.push('\n');
                }
                result.push_str(line);
            } else if line_num != curr_line {
                result.clear()
            } else if !result.is_empty() {
                break;
            }
        } else {
            if line.trim().starts_with("//") {
                result.push_str(line);
                result.push('\n');
            } else {
                break;
            }
        }
    }

    result
}
