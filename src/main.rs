use crate::parser::*;
use std::io::{stdin, stdout, Stdout, Write};
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::time::{Duration, Instant};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{clear, color, cursor, style};

pub mod parser;

static TIME_WINDOWS: [i32; 8] = [25, 5, 25, 5, 25, 5, 25, 15];

#[derive(Debug, PartialEq)]
enum TimerState {
    Idle,
    Running,
    Paused,
}

#[derive(Debug)]
struct Timer {
    start: Instant,
    curr_time_window: usize,
    duration: u64,
    //maybe use enum here for timer state instead of just bool
    running: TimerState,
}

// TODO put this into impl
fn start_timer(timer: &mut Timer) {
    timer.start = Instant::now();
    timer.duration = TIME_WINDOWS[timer.curr_time_window] as u64;
    timer.running = TimerState::Running;
}

fn pause_timer(timer: &mut Timer) {
    timer.duration = timer.duration - timer.start.elapsed().as_secs();
    timer.running = TimerState::Paused;
}

fn resume_timer(timer: &mut Timer) {
    timer.start = Instant::now();
    timer.running = TimerState::Running;
}

// TODO text edit
// TODO create new items
// TODO timer

fn print_list(stdout: &mut RawTerminal<Stdout>, curr_todo: usize, todos: &[Item], timer: &Timer) {
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

    let (x, _) = termion::terminal_size().unwrap();
    cursor_position += 1;
    if timer.running == TimerState::Running {
        write!(
            stdout,
            "{}{}{}",
            cursor::Goto((x / 2) - 2, 1),
            timer.duration - timer.start.elapsed().as_secs(),
            cursor::Goto(1, cursor_position)
        )
        .unwrap();
    } else {
        write!(
            stdout,
            "{}{}{}",
            cursor::Goto((x / 2) - 2, 1),
            timer.duration,
            cursor::Goto(1, cursor_position)
        )
        .unwrap();
    }

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

fn take_input(input: Sender<Key>) {
    let stdin = stdin();
    for c in stdin.keys() {
        let _ = input.send(c.unwrap());
    }
}

fn main() {
    let mut todos: Vec<Item> = Vec::new();
    parse_todos(&mut todos);
    let mut stdout = stdout().into_raw_mode().unwrap();
    let mut timer = Timer {
        start: Instant::now(),
        curr_time_window: 0,
        duration: 0,
        running: TimerState::Idle,
    };
    let mut curr_todo = 0;
    print_list(&mut stdout, curr_todo, &todos, &timer);
    let (input_sender, input_receiver) = channel();
    thread::spawn(|| take_input(input_sender));
    'render: loop {
        for c in input_receiver.try_iter() {
            match c {
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
                    break 'render;
                }
                Key::Char('\n') => match todos[curr_todo].status {
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
                Key::Char(' ') => match timer.running {
                    TimerState::Idle => start_timer(&mut timer),
                    TimerState::Running => pause_timer(&mut timer),
                    TimerState::Paused => resume_timer(&mut timer),
                },
                _ => {}
            }
        }
        print_list(&mut stdout, curr_todo, &todos, &timer);
        thread::sleep(Duration::from_millis(33));
    }
}
