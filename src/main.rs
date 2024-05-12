use parser::{Item, Status};
use std::io::{self, Stdout, Write};
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::{Duration, Instant};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{clear, color, cursor, style};

mod parser;

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
    running: TimerState,
}

// TODO write timer start/stop if curr is mod of 2
impl Timer {
    fn new() -> Timer {
        Timer {
            start: Instant::now(),
            curr_time_window: 0,
            duration: 0,
            running: TimerState::Idle,
        }
    }

    fn start(&mut self) {
        self.start = Instant::now();
        self.duration = TIME_WINDOWS[self.curr_time_window] as u64;
        self.running = TimerState::Running;
    }

    fn pause(&mut self) {
        self.duration -= self.start.elapsed().as_secs();
        self.running = TimerState::Paused;
    }

    fn resume(&mut self) {
        self.start = Instant::now();
        self.running = TimerState::Running;
    }

    fn check(&mut self) {
        if self.running == TimerState::Running
            && (self.duration - self.start.elapsed().as_secs()) < 1
        {
            self.running = TimerState::Idle;
            self.curr_time_window = (self.curr_time_window + 1) % TIME_WINDOWS.len();
            self.duration = TIME_WINDOWS[self.curr_time_window] as u64;
        }
    }
    fn reset(&mut self) {
        if self.running != TimerState::Running {
            self.curr_time_window = 0;
            self.running = TimerState::Idle;
        }
    }
}

// TODO write README
// TODO use lib.rs for implementation and use main for execution
// TODO reading logbook
// TODO writing clock-out
// TODO text edit
// TODO create new items
// TODO display time used
// TODO ascii clock
// TODO alarm sound
// TODO notification system
// TODO config for timers
fn get_clock_str(curr_time_window: usize, curr_time: u64) -> String {
    match curr_time_window {
        0..=1 => format!("@ # {curr_time} # #"),
        2..=3 => format!("# @ {curr_time} # #"),
        4..=5 => format!("# # {curr_time} @ #"),
        6..=7 => format!("# # {curr_time} # @"),
        _ => format!("# # {curr_time} # #"),
    }
}

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
    let curr_time = match timer.running {
        TimerState::Running => timer.duration - timer.start.elapsed().as_secs(),
        _ => timer.duration,
    };

    let clock_str = get_clock_str(timer.curr_time_window, curr_time);
    write!(
        stdout,
        "{}{}{}",
        cursor::Goto((x / 2) - 6, 1),
        clock_str,
        cursor::Goto(1, cursor_position)
    )
    .unwrap();

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

fn list_up(list: &[Item], list_curr: &mut usize) {
    if !list.is_empty() {
        if *list_curr > 0 {
            *list_curr -= 1;
        } else {
            *list_curr = list.len() - 1;
        }
    }
}

fn list_down(list: &[Item], list_curr: &mut usize) {
    if !list.is_empty() {
        if *list_curr == list.len() - 1 {
            *list_curr = 0;
        } else {
            *list_curr += 1;
        }
    }
}

fn take_input(input: Sender<Key>) {
    let stdin = io::stdin();
    for c in stdin.keys() {
        let _ = input.send(c.unwrap());
    }
}

fn main() {
    let mut todos: Vec<Item> = Vec::new();
    parser::parse_todos(&mut todos);
    let mut stdout = io::stdout().into_raw_mode().unwrap();
    let mut timer = Timer::new();
    let mut curr_todo = 0;
    print_list(&mut stdout, curr_todo, &todos, &timer);
    let (input_sender, input_receiver) = mpsc::channel();
    thread::spawn(|| take_input(input_sender));
    'render: loop {
        timer.check();
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
                        parser::write_todo_state(&todos[curr_todo]);
                        parser::parse_todos(&mut todos);
                    }
                    Status::Todo => {
                        todos[curr_todo].status = Status::Done;
                        parser::write_todo_state(&todos[curr_todo]);
                        parser::parse_todos(&mut todos);
                    }
                },
                Key::Char('j') => list_down(&todos, &mut curr_todo),
                Key::Char('k') => list_up(&todos, &mut curr_todo),
                Key::Char('r') => timer.reset(),
                Key::Char(' ') => match timer.running {
                    TimerState::Idle => {
                        timer.start();
                        parser::write_clock_in(&todos[curr_todo]).unwrap();
                        parser::parse_todos(&mut todos);
                    }
                    TimerState::Running => timer.pause(),
                    TimerState::Paused => {
                        timer.resume();
                        parser::write_clock_in(&todos[curr_todo]).unwrap();
                        parser::parse_todos(&mut todos);
                    }
                },
                _ => {}
            }
        }
        print_list(&mut stdout, curr_todo, &todos, &timer);
        thread::sleep(Duration::from_millis(33));
    }
}
