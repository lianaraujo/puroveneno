use itertools::Itertools;
use regex::Regex;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

#[derive(Debug)]
pub enum Status {
    Todo,
    Done,
}

#[derive(Debug)]
pub struct Item {
    pub heading: usize,
    pub text: String,
    pub status: Status,
}

fn parse_todo(heading: &str) -> Option<Status> {
    let todo_reg: Regex = Regex::new(r"\*.TODO").unwrap();
    let done_reg: Regex = Regex::new(r"\*.DONE").unwrap();
    if todo_reg.is_match(heading) {
        return Some(Status::Todo);
    }
    if done_reg.is_match(heading) {
        return Some(Status::Done);
    }
    None
}

pub fn parse_todos(todos: &mut Vec<Item>) -> io::Result<()> {
    let heading_reg: Regex = Regex::new(r"\*.*").unwrap();
    let file = File::open("notas.org")?;
    let mut current_todo: Item;
    for line in BufReader::new(file).lines() {
        match line {
            Ok(line) => {
                if heading_reg.is_match(&line) {
                    match parse_todo(&line) {
                        Some(Status::Todo) => {
                            let mut split = line.split(" ");
                            current_todo = Item {
                                heading: String::from(split.next().unwrap()).chars().count(),
                                status: Status::Todo,
                                text: String::from(split.join(" ").strip_prefix("TODO ").unwrap()),
                            };
                            todos.push(current_todo)
                        }
                        Some(Status::Done) => {
                            let mut split = line.split(" ");
                            current_todo = Item {
                                heading: String::from(split.next().unwrap()).chars().count(),
                                status: Status::Done,
                                text: String::from(split.join(" ").strip_prefix("DONE ").unwrap()),
                            };
                            todos.push(current_todo)
                        }
                        None => {
                            continue;
                        }
                    }
                } else {
                    // look for properties
                    todo!()
                }
            }
            Err(_line) => continue,
        }
    }
    Ok(())
}
