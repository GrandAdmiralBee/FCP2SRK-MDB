use colored::*;
use regex::Regex;

use std::collections::HashMap;

use super::FCP;

pub fn join_log_lines(buffer: &String) -> String {
    let mut res = String::new();

    let mut split_lines = String::new();
    let mut joined = true;

    let mut curr_line_num = 0;
    let mut new_line_num = 0;

    for line in buffer.lines() {
        curr_line_num += 1;
        if !joined {
            split_lines.push_str(&format!(" {}", line.trim()));

            println!("{}", line.yellow());

            if line.ends_with(";") {
                res.push_str(&split_lines);
                res.push('\n');
                println!(
                    "------ Above is replaced with (now on line {})------\n{}",
                    new_line_num, split_lines
                );
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
                println!(
                    "------ Multiple-line logs starting on line {} (now on line {}) ------\n{}",
                    curr_line_num,
                    new_line_num,
                    line.yellow()
                );
                continue;
            }

            println!(
                "------ One-line log on line {} (now on line {}) ------\n{}",
                curr_line_num, new_line_num, line
            );

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
    line_num: u32,
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
        println!(
            "--------- For line {line_num} -------\n{}\n--------- found comments\n{}",
            line.trim(),
            commented_lines.trim()
        );

        logger_map.iter().for_each(|(k, fcp)| {
            if commented_lines.contains(k) {
                let codes = loggers.get(fcp).unwrap();
                let opt = codes.get(err);

                if opt.is_none() {
                    println!("No error code for {err} in {k}");
                } else {
                    mdb_match = opt.unwrap().to_string();
                    println!("Got {} for {} code in {}", mdb_match, err, k);
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

        println!("{base_str}");

        loop {
            let mut buffer = String::new();
            std::io::stdin().read_line(&mut buffer).unwrap();
            let buffer = buffer.trim();

            match buffer.trim().parse::<usize>() {
                Ok(num) => {
                    if num > fcp_vec.len() {
                        println!("wrong input! try again:");
                        continue;
                    } else {
                        let fcp = fcp_vec[num - 1];
                        let codes = loggers.get(&fcp).unwrap();
                        let opt = codes.get(err);

                        if opt.is_none() {
                            println!("No error code for {err} in {}", fcp.to_str());
                        } else {
                            println!("Got {} for {} code in {}", mdb_match, err, fcp.to_str());
                            mdb_match = opt.unwrap().to_string();
                            break;
                        }
                    }
                }
                Err(..) => {
                    println!("wrong input! try again:");
                    continue;
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

    println!(
        "------- Replacing log on line {} --------\n{}\n{}",
        line_num,
        line.yellow(),
        new_line.green()
    );
    //println!(
    //    "------- Replace \n{}\n------ In line {}\n-------- With \n{}\n\n (yes/no):",
    //    line.green(),
    //    line_num,
    //    new_line.blue()
    //);
    //let mut answer = String::new();
    //std::io::stdin().read_line(&mut answer).unwrap();
    //answer = answer.trim().to_string();
    //if "no".contains(&answer) {
    //    return None;
    //}

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
