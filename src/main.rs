use crate::parser::*;
use std::io::{stdin, stdout, Stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{clear, color, cursor, style};
pub mod parser;

fn print_list(stdout: &mut RawTerminal<Stdout>, curr_todo: usize, todos: &[Item]) {
    let mut cursor_position = 1;
    write!(
        stdout,
        "{}{}{}",
        clear::All,
        cursor::Goto(1, 1),
        cursor::Hide
    )
    .unwrap();
    stdout.flush().unwrap();

    for (index, todo) in todos.iter().enumerate() {
        cursor_position += 1;
        match todo.status {
            Status::Todo => {
                write!(stdout, "{}TODO ", color::Fg(color::Red)).unwrap();
            }
            Status::Done => {
                write!(stdout, "{}DONE ", color::Fg(color::Green)).unwrap();
            }
        }
        if index == curr_todo {
            write!(
                stdout,
                "{}{}{}{}{}{}",
                color::Bg(color::White),
                color::Fg(color::Black),
                todo.text,
                cursor::Goto(1, cursor_position),
                color::Bg(color::Reset),
                color::Fg(color::Reset)
            )
            .unwrap();
        } else {
            write!(
                stdout,
                "{}{}{}{}{}{}",
                color::Bg(color::Black),
                color::Fg(color::White),
                todo.text,
                cursor::Goto(1, cursor_position),
                color::Bg(color::Reset),
                color::Fg(color::Reset)
            )
            .unwrap();
        }
        stdout.flush().unwrap();
    }
}

fn list_up(list: &Vec<Item>, list_curr: &mut usize) {
    if !list.is_empty() {
        if *list_curr > 0 {
            *list_curr -= 1;
        } else {
            *list_curr = list.len() - 1;
        }
    }
}

fn list_down(list: &Vec<Item>, list_curr: &mut usize) {
    if !list.is_empty() {
        if *list_curr == list.len() - 1 {
            *list_curr = 0;
        } else {
            *list_curr += 1;
        }
    }
}

fn main() {
    let mut todos: Vec<Item> = Vec::new();
    parse_todos(&mut todos);
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();

    let mut curr_todo = 0;
    print_list(&mut stdout, curr_todo, &todos);

    for c in stdin.keys() {
        match c.unwrap() {
            Key::Char('q') => {
                write!(
                    stdout,
                    "{}{}{}{}",
                    clear::All,
                    style::Reset,
                    cursor::Goto(1, 1),
                    cursor::Show
                )
                .unwrap();
                stdout.flush().unwrap();
                break;
            }
            Key::Char('\n') => match todos[curr_todo].status {
                // TODO write files when toggling
                Status::Done => {
                    todos[curr_todo].status = Status::Todo;
                    let _ = write_todo_state(&todos[curr_todo]);
                }
                Status::Todo => {
                    todos[curr_todo].status = Status::Done;
                    let _ = write_todo_state(&todos[curr_todo]);
                }
            },
            Key::Char('j') => list_down(&todos, &mut curr_todo),
            Key::Char('k') => list_up(&todos, &mut curr_todo),
            Key::Char(' ') => {
                //TODO start stop
            }
            _ => {}
        }
        print_list(&mut stdout, curr_todo, &todos)
    }
}
