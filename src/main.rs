extern crate regex;
extern crate chrono;

use chrono::{Datelike, Timelike};
use chrono::NaiveDate;
use chrono::duration::Duration;
use chrono::naive::datetime::NaiveDateTime;
use chrono::naive::time::NaiveTime;
use regex::Regex;

use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufRead, Write};
use std::path::PathBuf;
use std::process;

static RESTR: &'static str = r"\[(?P<start_datetime>[^\[\]]+)\]--\[(?P<end_datetime>[^\[\]]+)\]\s+=>\s+(?P<duration>\d+:\d{2})";

pub fn main() {
    let re: Regex = Regex::new(RESTR).unwrap();

    let home = match env::var("HOME") {
        Ok(value) => value,
        Err(_) => "".to_string(),
    };

    let mut filename = PathBuf::from(&home);
    filename.push("org");
    filename.push("proc");
    filename.set_extension("org");

    let argv: Vec<_> = env::args().collect();
    if argv.len() > 1 {
        filename = PathBuf::from(&argv[1]);
    }

    let mut map: HashMap<NaiveDate, Duration> = HashMap::new();
    let file = File::open(filename).unwrap();

    for line in BufReader::new(file).lines() {
        match line {
            Ok(text) => {
                if re.is_match(text.as_ref()) {
                    let start_datetime = match capture_name(&re, text.as_ref(), "start_datetime") {
                        Some(start_datetime) => start_datetime,
                        None => {
                            let _result = writeln!(&mut std::io::stderr(), "start_datetime is None");
                            process::exit(1);
                        }
                    };
                    let start_datetime = match NaiveDateTime::parse_from_str(start_datetime, "%Y-%m-%d %a %H:%M") {
                        Ok(start_datetime) => start_datetime,
                        Err(e) => {
                            let _result = writeln!(&mut std::io::stderr(),
                                                   "failed to parse start_datetime: {}",
                                                   start_datetime);
                            let _result = writeln!(&mut std::io::stderr(), "{}", e.description());
                            process::exit(1);
                        }
                    };
                    let end_datetime = match capture_name(&re, text.as_ref(), "end_datetime") {
                        Some(end_datetime) => end_datetime,
                        None => {
                            let _result = writeln!(&mut std::io::stderr(), "end_datetime is None");
                            process::exit(1);
                        }
                    };
                    let end_datetime = match NaiveDateTime::parse_from_str(end_datetime, "%Y-%m-%d %a %H:%M") {
                        Ok(end_datetime) => end_datetime,
                        Err(e) => {
                            let _result = writeln!(&mut std::io::stderr(),
                                                   "failed to parse end_datetime: {}",
                                                   end_datetime);
                            let _result = writeln!(&mut std::io::stderr(), "{}", e.description());
                            process::exit(1);
                        }
                    };

                    let date = start_datetime.date();
                    if format!("{}", start_datetime.format("%y-%m-%d")) != format!("{}", end_datetime.format("%y-%m-%d")) {
                        let _result = writeln!(&mut std::io::stderr(),
                                               "start datetime is not equivalent to end datetime");
                        process::exit(1);
                    }

                    let duration = match capture_name(&re, text.as_ref(), "duration") {
                        Some(duration) => duration,
                        None => {
                            let _result = writeln!(&mut std::io::stderr(), "duration is None");
                            process::exit(1);
                        }
                    };
                    let duration = match NaiveTime::parse_from_str(duration, "%_H:%M") {
                        Ok(duration) => duration,
                        Err(e) => {
                            let _result = writeln!(&mut std::io::stderr(),
                                                   "failed to parse duration: {}",
                                                   duration);
                            let _result = writeln!(&mut std::io::stderr(), "{}", e.description());
                            process::exit(1);
                        }
                    };
                    let mut duration = duration - NaiveTime::from_hms(0, 0, 0);

                    // Validate duration
                    if duration != (end_datetime - start_datetime) {
                        println!("start_datetime: {}", start_datetime);
                        println!("end_datetime: {}", end_datetime);
                        let _result = writeln!(&mut std::io::stderr(),
                                               "invalid duration: duration: {}",
                                               duration);
                        process::exit(1);
                    }

                    // Insert and update date value
                    if map.contains_key(&date) {
                        match map.get(&date) {
                            Some(value) => {
                                duration = duration + *value;
                            }
                            None => {
                                let _result = writeln!(&mut std::io::stderr(), "value is None");
                                process::exit(1);
                            }
                        }
                        map.remove(&date);
                        map.insert(date, duration);
                    } else {
                        map.insert(date, duration);
                    }
                }
            }
            Err(e) => {
                let _result = writeln!(&mut std::io::stderr(),
                                       "failed to get line: {}",
                                       e.description());
                process::exit(1);
            }
        }
    }

    // Print date, working time
    let mut sum = Duration::seconds(0);
    for x in 0..32 {
        for (date, &duration) in &map {
            if date.day() == x as u32 {
                let time = NaiveTime::from_hms(0, 0, 0) + duration;
                println!("{}: {:2}h{:2}m", date.format("%Y-%m-%d %a"), time.hour(), time.minute());
                sum = sum + duration;
            }
        }
    }
    let datetime = NaiveDateTime::new(NaiveDate::from_ymd(1970,1,1), NaiveTime::from_hms(0, 0, 0)) + sum;
    println!("sum:            {:2}d{:2}h{:2}m", datetime.day()-1, datetime.hour(), datetime.minute());
    println!("goal:           {:2}d{:2}h 0m", map.len() * 8 / 24, map.len() * 8 % 24);
}

fn capture_name<'t>(re: &'t Regex, text: &'t str, name: &str) -> Option<&'t str> {
    match re.captures(text) {
        Some(caps) => {
            match caps.name(name) {
                Some(cap) => Some(cap.as_str()),
                None => None,
            }
        }
        None => None,
    }
}
