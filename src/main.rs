#[macro_use]
extern crate clap;
extern crate chrono;
extern crate dirs;
extern crate open;

use chrono::prelude::*;
use clap::{Arg, SubCommand};
use std::path::PathBuf;

use std::collections::HashMap;
use std::fs;
use std::fs::OpenOptions;
use std::io::BufRead;
use std::io::BufReader;

use std::io::Write;

fn main() {
    let matches = app_from_crate!()
    	.template("{bin} {version}\n{author}\n\n{about}\n\nUSAGE:\n    {usage}\n\nFLAGS:\n{flags}\n\nSUBCOMMANDS:\n{subcommands}")
    	.subcommand(SubCommand::with_name("cur").about("Get current ideas"))
    	.subcommand(SubCommand::with_name("edit").about("Edit current ideas"))
    	.subcommand(SubCommand::with_name("idea")
    		.about("add an idea to your current ideas")
    		.arg(Arg::with_name("idea").index(1)),
    		)
        .subcommand(SubCommand::with_name("log")
        	.about("Print ideas log"))
        .subcommand(SubCommand::with_name("entry")
        	.about("Add log entry for a current idea")
        	.arg(Arg::with_name("idea").index(1))
        	.arg(Arg::with_name("content").index(2)),
        	)
        .subcommand(SubCommand::with_name("hist")
        	.about("see log entries for one par idea")
        	.arg(Arg::with_name("idea").index(1)),
        	)
        .get_matches();

    let chasm = Chasm::new();

    match matches.subcommand() {
        ("cur", Some(_sub_matches)) => {
            let ideas = chasm.cur();
            println!("ideas: -------------------------------------");
            for idea in ideas {
                println!("	{:?}", idea);
            }
        }
        ("edit", Some(_sub_matches)) => {
            chasm.edit()
        }
        ("idea", Some(sub_matches)) => {
            let idea = sub_matches.value_of("idea");

            match idea {
                Some(idea) => chasm.idea(idea),
                None => println!("no matches"),
            }
        }
        ("entry", Some(sub_matches)) => {
            let idea = sub_matches.value_of("idea");
            let content = sub_matches.value_of("content");

            if idea.is_some() && content.is_some() {
                chasm.entry(idea.unwrap(), content.unwrap())
            } else {
                println!("input does not match!")
            }
        }
        ("log", Some(_sub_matches)) => {
            println!("log: 	-------------------------------------");
            let log = chasm.create_log();
            for entry in log {
                println!("	| {:?} ------ {:?}", entry.time, entry.thoughts);
            }
        }
        ("hist", Some(sub_matches)) => {
            let idea = sub_matches.value_of("idea");

            match idea {
                Some(idea) => {
                    println!("{:?}: -------------------------------------", idea);
                    let log = chasm.create_log();
                    for entry in log {
                        if entry.idea == idea {
                            println!("	| {:?} ------ {:?}", entry.time, entry.thoughts);
                        }
                    }
                }
                None => println!("no match"),
            }
        }
        _ => {
            println!("not a valid subcommand");
        }
    }
}

struct Entry {
    idea: String,
    thoughts: String,
    time: Date<Local>,
}

struct Idea {
    description: String,
    entries: Vec<Entry>,
}

struct Chasm {
    ideas_file: PathBuf,
    log_file: PathBuf,
    log: Vec<Entry>,
    dictionary: HashMap<String, Idea>,
}

impl Chasm {
    fn new() -> Chasm {
        let mut home_dir = dirs::home_dir().unwrap();

        home_dir.push(".chasmic");

        if !home_dir.is_dir() {
            println!("Chasmic - document the ideas that are on top of your head~");
            println!("");
            fs::create_dir(&home_dir).unwrap();
        }

        let mut log_file = home_dir.clone();
        log_file.push("log");

        if !log_file.is_file() {
            fs::File::create(&log_file).unwrap();
            println!("generated a log file at {}", log_file.to_str().unwrap());
        }

        let mut ideas_file = home_dir.clone();
        ideas_file.push("ideas");

        if !ideas_file.is_file() {
            fs::File::create(&ideas_file).unwrap();
            println!(
                "generated a file for holding your current ideas at {}",
                ideas_file.to_str().unwrap()
            );
        }

        Chasm {
            ideas_file,
            log_file,
            log: vec![],
            dictionary: HashMap::new(),
        }
    }

    fn create_log(&self) -> Vec<Entry> {
        let mut entries = vec![];

        let f = fs::File::open(&self.log_file).unwrap();
        let file = BufReader::new(&f);

        for line in file.lines() {
            let unwrapped = line.unwrap();

            let split = unwrapped.split(" | ");
            let parts: Vec<&str> = split.collect();

            let time = DateTime::parse_from_rfc3339(parts[0]).unwrap();

            let entry = Entry {
                idea: parts[1].to_string(),
                thoughts: parts[2].to_string(),
                time: time.with_timezone(&Local).date(),
            };

            entries.push(entry)
        }

        entries
    }

    fn create_dict(&self) -> HashMap<String, Idea> {
        let mut dictionary: HashMap<String, Idea> = HashMap::new();

        let f = fs::File::open(&self.log_file).unwrap();
        let file = BufReader::new(&f);

        for line in file.lines() {
            let unwrapped = line.unwrap();

            let split = unwrapped.split(" | ");
            let parts: Vec<&str> = split.collect();

            let time = DateTime::parse_from_rfc3339(parts[0]).unwrap();

            let string = parts[1].to_string();

            if dictionary.contains_key(&string) {
                let string = parts[1].clone();
                let idea = dictionary.get_mut(string);

                let entry = Entry {
                    idea: parts[1].to_string(),
                    thoughts: parts[2].to_string(),
                    time: time.with_timezone(&Local).date(),
                };

                idea.unwrap().entries.push(entry)
            } else {
                let entry = Entry {
                    idea: parts[1].to_string(),
                    thoughts: parts[2].to_string(),
                    time: time.with_timezone(&Local).date(),
                };

                let mut entries = vec![];
                entries.push(entry);

                let idea = Idea {
                    description: parts[1].to_string(),

                    entries,
                };

                let string = parts[1].to_string();
                dictionary.insert(string, idea);
            }
        }

        dictionary
    }

    fn create_dictionary(&self) {}

    fn edit(&self) {
        open::that(&self.ideas_file);
    }

    fn idea(&self, idea: &str) {
        let file = OpenOptions::new()
            .append(true)
            .open(&self.ideas_file)
            .unwrap();

        write!(&file, "{}\n", idea,);
    }

    fn entry(&self, idea: &str, text: &str) {
        let file = OpenOptions::new()
            .append(true)
            .open(&self.log_file)
            .unwrap();
        let time = Local::now();

        write!(&file, "{:?} | {} | {}\n", time, idea, text,);
    }

    fn cur(&self) -> Vec<String> {
        let f = fs::File::open(&self.ideas_file).unwrap();
        let file = BufReader::new(&f);

        let mut ideas = vec![];

        for line in file.lines() {
            ideas.push(line.unwrap());
        }

        ideas
    }
}
