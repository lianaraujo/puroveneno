use chrono::prelude::*;
use itertools::Itertools;
use regex::Regex;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Lines, Result as IoResult, Write};
use std::path::Path;

// could save text inside the enum maybe
#[derive(Debug)]
pub enum Status {
    Todo,
    Done,
}

#[derive(Debug)]
pub struct Item {
    pub line_number: usize,
    pub text: String,
    pub status: Status,
}

fn parse_todo(heading: &str) -> Option<Status> {
    let todo_reg: Regex = Regex::new(r"\*\sTODO").unwrap();
    let done_reg: Regex = Regex::new(r"\*\sDONE").unwrap();
    if todo_reg.is_match(heading) {
        return Some(Status::Todo);
    }
    if done_reg.is_match(heading) {
        return Some(Status::Done);
    }
    None
}

fn read_lines<P>(filename: P) -> IoResult<Lines<BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename).expect("Can't fiind file");
    Ok(BufReader::new(file).lines())
}

pub fn write_todo_state(todo: &Item) {
    let mut f = OpenOptions::new()
        .append(true)
        .create(true)
        .open("./tmp")
        .expect("Unable to open file");

    if let Ok(lines) = read_lines("./notas.org") {
        for (index, line) in lines.enumerate() {
            if let Ok(mut line) = line {
                if todo.line_number == index {
                    match todo.status {
                        Status::Todo => line = line.replace("DONE", "TODO"),
                        Status::Done => line = line.replace("TODO", "DONE"),
                    };
                }
                writeln!(&mut f, "{}", line).expect("Unable to write");
            }
        }
    }
    fs::rename("tmp", "notas.org").expect("Error while overwriting");
}

pub fn write_clock_in(todo: &Item) -> IoResult<()> {
    let mut f = OpenOptions::new()
        .append(true)
        .create(true)
        .open("./tmp")
        .expect("Unable to open file");
    let dt = Local::now();

    if let Ok(lines) = read_lines("./notas.org") {
        let mut linesiter = lines.enumerate().peekable();
        while let Some((index, line)) = linesiter.next() {
            if let Ok(line) = line {
                if (todo.line_number + 1) == index {
                    if line.contains(":LOGBOOK:") {
                        writeln!(&mut f, "{}", line)?;
                        writeln!(&mut f, "  CLOCK: {}", dt.format("[%Y-%m-%d %a %H:%M]"))?;
                    } else {
                        writeln!(&mut f, "  :LOGBOOK:")?;
                        writeln!(&mut f, "  CLOCK: {}", dt.format("[%Y-%m-%d %a %H:%M]"))?;
                        writeln!(&mut f, "  :END:")?;
                        writeln!(&mut f, "{}", line)?;
                    }
                } else if todo.line_number == index && linesiter.peek().is_none() {
                    writeln!(&mut f, "{}", line)?;
                    writeln!(&mut f, "  :LOGBOOK:")?;
                    writeln!(&mut f, "  CLOCK: {}", dt.format("[%Y-%m-%d %a %H:%M]"))?;
                    writeln!(&mut f, "  :END:")?;
                } else {
                    writeln!(&mut f, "{}", line)?;
                }
            }
        }
    }
    fs::rename("tmp", "notas.org")?;
    Ok(())
}
pub fn parse_todos(todos: &mut Vec<Item>) {
    let heading_reg: Regex = Regex::new(r"\*.*").unwrap();
    // TODO pass file as arg and have a default file
    let mut new_todos: Vec<Item> = Vec::new();
    let mut current_todo: Item;
    if let Ok(lines) = read_lines("./notas.org") {
        for (index, line) in lines.enumerate() {
            if let Ok(line) = line {
                if heading_reg.is_match(&line) {
                    if let Some(status) = parse_todo(&line) {
                        let prefix = match status {
                            Status::Todo => "TODO ",
                            Status::Done => "DONE ",
                        };
                        let mut split = line.split(' ');
                        split.next();
                        current_todo = Item {
                            line_number: index,
                            status,
                            text: String::from(split.join(" ").strip_prefix(prefix).unwrap()),
                        };
                        new_todos.push(current_todo)
                    }
                } else {
                    // TODO look for properties
                    continue;
                }
            }
        }
    }
    *todos = new_todos;
}
