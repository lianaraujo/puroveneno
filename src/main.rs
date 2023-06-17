use std::io::{stdin, stdout, Stdout, Write};
use termion::color::{AnsiValue, Color, Reset};
use termion::cursor::Goto;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{clear, color, cursor, style};

enum Status {
    TODO,
    DONE,
}
struct Item {
    text: String,
    status: Status,
}

fn print_list(stdout: &mut RawTerminal<Stdout>, curr_todo: usize, todos: &Vec<Item>) {
    let mut cursor = 1;
    write!(stdout, "{}{}{}", clear::All, Goto(1, 1), cursor::Hide).unwrap();
    stdout.flush().unwrap();

    for (index, todo) in todos.iter().enumerate() {
        cursor += 1;
        match todo.status {
            Status::TODO => {
                write!(stdout, "{}{}", color::Fg(color::Red), "TODO ").unwrap();
            }
            Status::DONE => {
                write!(stdout, "{}{}", color::Fg(color::Green), "DONE ").unwrap();
            }
        }
        if index == curr_todo {
            write!(
                stdout,
                "{}{}{}{}{}{}",
                color::Bg(color::White),
                color::Fg(color::Black),
                todo.text,
                Goto(1, cursor),
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
                Goto(1, cursor),
                color::Bg(color::Reset),
                color::Fg(color::Reset)
            )
            .unwrap();
        }
        stdout.flush().unwrap();
    }
}

fn main() {
    let mut todos: Vec<Item> = vec![
        Item {
            text: "Ass".to_string(),
            status: Status::TODO,
        },
        Item {
            text: "DoubleAss".to_string(),
            status: Status::TODO,
        },
        Item {
            text: "Triple Ass".to_string(),
            status: Status::DONE,
        },
    ];
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();

    let mut curr_todo = 0;
    print_list(&mut stdout, curr_todo, &todos);

    for c in stdin.keys() {
        match c.unwrap() {
            Key::Char('q') => break,
            Key::Char('j') => curr_todo += 1,
            Key::Char('k') => curr_todo -= 1,
            Key::Char(' ') => match todos[curr_todo].status {
                Status::DONE => todos[curr_todo].status = Status::TODO,
                Status::TODO => todos[curr_todo].status = Status::DONE,
            },
            _ => {}
        }
        print_list(&mut stdout, curr_todo, &todos)
    }
}
