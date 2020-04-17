use std::cmp::{max, min};
use std::fs::File;
use std::io::{prelude::*, stdin, stdout, BufReader, Stdout, Write};
use std::path::PathBuf;
use termion::style::{Invert, Reset};
use termion::{cursor::Goto, event::Key, input::TermRead, raw::IntoRawMode};

fn to_str(s: &Vec<char>) -> String {
    s.iter().collect()
}

fn to_vec(s: &str) -> Vec<char> {
    s.chars().collect()
}

enum State {
    Femto,
    Cmd((Command, LineBuffer)),
}

enum Command {
    Open,
    Save,
}

trait Buffer {
    fn push(&mut self, c: char);
    fn backspace(&mut self);
    fn delete(&mut self);
    fn move_caret(&mut self, row: i32, col: i32);
}

struct Editor {
    file_buffer: FileBuffer,
    state: State,
    message: Option<String>,
}

impl Editor {
    fn new() -> Self {
        Self {
            file_buffer: FileBuffer::new(),
            state: State::Femto,
            message: None,
        }
    }

    fn buffer(&mut self) -> &mut dyn Buffer {
        match &mut self.state {
            State::Femto => &mut self.file_buffer,
            State::Cmd((_, buffer)) => buffer,
        }
    }

    fn push(&mut self, c: char) {
        if c == '\n' {
            match &self.state {
                State::Femto => self.buffer().push(c),
                State::Cmd((state, buffer)) => {
                    let line = buffer.line.clone();
                    match state {
                        Command::Open => self.open(PathBuf::from(&to_str(&line))),
                        Command::Save => self.save(PathBuf::from(&to_str(&line))),
                    }
                }
            }
        } else {
            self.buffer().push(c);
        }
    }

    fn start_open(&mut self) {
        self.state = State::Cmd((Command::Open, LineBuffer::default()));
    }

    fn open(&mut self, path: PathBuf) {
        match self.file_buffer.load(path.clone()) {
            Ok(_) => self.exit_command(),
            Err(err) => self.show_message(err.to_string()),
        }
    }

    fn start_save(&mut self) {
        let mut buffer = LineBuffer::default();
        buffer.line = self.file_buffer.path.to_str().unwrap().chars().collect();
        buffer.col = buffer.line.len();
        self.state = State::Cmd((Command::Save, buffer));
    }

    fn save(&mut self, path: PathBuf) {
        match self.file_buffer.save(path.clone()) {
            Ok(_) => self.exit_command(),
            Err(err) => self.show_message(err.to_string()),
        }
    }

    fn prompt(&self) -> (&str, String, usize) {
        match &self.state {
            State::Femto => match &self.message {
                Some(message) => ("", message.clone(), 0),
                None => ("femto", String::new(), 0),
            },
            State::Cmd((state, buf)) => match state {
                Command::Open => ("Open file at: ", to_str(&buf.line), buf.col),
                Command::Save => ("Save file at: ", to_str(&buf.line), buf.col),
            },
        }
    }

    fn show_message(&mut self, message: String) {
        self.exit_command();
        self.message = Some(message);
    }

    fn exit_command(&mut self) {
        self.state = State::Femto;
    }
}

struct FileBuffer {
    row_offset: usize,
    col_offset: usize,
    row: usize,
    col: usize,
    path: PathBuf,
    lines: Vec<Vec<char>>,
}

impl FileBuffer {
    fn new() -> Self {
        Self {
            row_offset: 0,
            col_offset: 0,
            row: 0,
            col: 0,
            path: PathBuf::default(),
            lines: vec![vec![]],
        }
    }

    fn line(&mut self) -> &mut Vec<char> {
        self.lines.get_mut(self.row).unwrap()
    }

    fn load(&mut self, path: PathBuf) -> Result<(), std::io::Error> {
        let file = File::open(path.clone())?;
        let converter = |l: Result<String, _>| to_vec(&l.unwrap());
        self.lines = BufReader::new(file).lines().map(converter).collect();
        self.path = path;
        self.row = 0;
        self.col = 0;
        Ok(())
    }

    fn save(&self, path: PathBuf) -> Result<(), std::io::Error> {
        let mut file = File::create(path.clone())?;
        for line in self.lines.iter() {
            writeln!(file, "{}", to_str(line)).unwrap();
        }
        Ok(())
    }
}

impl Buffer for FileBuffer {
    fn push(&mut self, c: char) {
        let (col, row) = (self.col, self.row);

        if c == '\n' {
            let new_line = self.line().drain(col..).collect();
            self.lines.insert(row + 1, new_line);
            self.move_caret(1, -(col as i32));
            return;
        }

        self.line().insert(col, c);
        self.move_caret(0, 1);
    }

    fn backspace(&mut self) {
        let (col, row) = (self.col, self.row);

        if col == 0 && row != 0 {
            let line = self.lines.remove(row);
            self.move_caret(-1, 0);
            let len = self.line().len() as i32 - col as i32;
            self.move_caret(0, len);
            self.line().extend(line.iter());
        } else if col != 0 {
            self.line().remove(col - 1);
            self.move_caret(0, -1);
        }
    }

    fn delete(&mut self) {
        let (col, row) = (self.col, self.row);

        if col == self.line().len() && row != self.lines.len() - 1 {
            let line = self.lines.remove(row + 1);
            self.line().extend(line.iter());
        } else if col != self.line().len() {
            self.line().remove(col);
        }
    }

    fn move_caret(&mut self, row: i32, col: i32) {
        let (w, h) = termion::terminal_size().expect("Unsupported terminal.");

        let num_lines = self.lines.len() as i32;
        self.row = min(max(self.row as i32 + row, 0), num_lines - 1) as usize;
        if self.row < self.row_offset {
            self.row_offset = self.row;
        } else if self.row > self.row_offset + (h as usize - 2) {
            self.row_offset = self.row - (h as usize - 2);
        }

        let line_len = self.line().len() as i32;
        self.col = min(max(self.col as i32 + col, 0), line_len) as usize;
        if self.col < self.col_offset {
            self.col_offset = self.col;
        } else if self.col > self.col_offset + (w as usize - 1) {
            self.col_offset = self.col - (w as usize - 1);
        }
    }
}

#[derive(Default)]
struct LineBuffer {
    col: usize,
    line: Vec<char>,
}

impl Buffer for LineBuffer {
    fn push(&mut self, c: char) {
        if c != '\n' {
            self.line.insert(self.col, c);
            self.move_caret(0, 1);
        }
    }

    fn backspace(&mut self) {
        if self.col != 0 {
            self.line.remove(self.col - 1);
            self.move_caret(0, -1);
        }
    }

    fn delete(&mut self) {
        if self.col != self.line.len() {
            self.line.remove(self.col);
        }
    }

    fn move_caret(&mut self, _: i32, col: i32) {
        let line_len = self.line.len() as i32;
        self.col = min(max(self.col as i32 + col, 0), line_len) as usize;
    }
}

fn main() {
    let mut stdout = stdout().into_raw_mode().expect("Unsupported terminal.");
    let mut editor = Editor::new();

    let mut args: Vec<String> = std::env::args().collect();
    if args.len() == 2 {
        editor.open(PathBuf::from(args.remove(1)));
    } else if args.len() > 2 {
        return println!("Error: too many arguments.\nusage: femto [FILE]");
    }

    loop {
        print_screen(&mut stdout, &mut editor);
        if handle_keys(&mut editor) {
            break;
        }
    }

    write!(stdout, "{}{}", termion::clear::All, Goto(1, 1)).unwrap();
    stdout.flush().unwrap();
}

fn print_screen(stdout: &mut Stdout, editor: &mut Editor) {
    let file_buf = &editor.file_buffer;
    let (roff, coff) = (file_buf.row_offset, file_buf.col_offset);
    let (r, c) = (file_buf.row + 1, file_buf.col + 1);
    let (w, h) = termion::terminal_size().expect("Unsupported terminal.");

    // Clear and start writing from origin
    write!(stdout, "{}{}", termion::clear::All, Goto(1, 1)).unwrap();

    for i in roff..(roff + h as usize - 1) {
        if i < file_buf.lines.len() {
            // Content
            let line = file_buf.lines.get(i).unwrap();

            if line.len() < coff {
                write!(stdout, "\n\r").unwrap();
                continue;
            }

            let part = &line[coff..min(coff + w as usize, line.len())];
            write!(stdout, "{}\n\r", to_str(&Vec::from(part))).unwrap();
        } else {
            // ~ as filler for parts of the window that are outside the buffer
            write!(stdout, "~\n\r").unwrap();
        }
    }

    // Status bar
    let (prompt, mut cmd, cmd_col) = editor.prompt();
    let bar_right = format!(" row: {}, col: {}", r, c);
    let avail = w as usize - bar_right.len() - prompt.len();
    let cmd_cur_pos = (prompt.len() + cmd_col + 1) as u16;
    let mut start = 0;

    let mut spacer = String::new();
    if cmd.len() > avail {
        start = min(cmd_col, cmd.len() - avail);
        cmd = cmd[start..(start + avail)].to_string();
    } else {
        spacer = " ".repeat(avail - cmd.len());
    }

    let bar = format!("{}{}{}{}", prompt, cmd, spacer, bar_right);
    write!(stdout, "{}{}{}", Invert, bar, Reset).unwrap();

    // Draw cursor on the right place
    match editor.state {
        State::Femto => write!(stdout, "{}", Goto((c - coff) as u16, (r - roff) as u16)).unwrap(),
        _ => write!(stdout, "{}", Goto(cmd_cur_pos - start as u16, h)).unwrap(),
    }
    // Ensure everything visible
    stdout.flush().unwrap();

    editor.message = None;
}

fn handle_keys(editor: &mut Editor) -> bool {
    let c = stdin().keys().next().unwrap();
    match c.unwrap() {
        Key::Char(c) => editor.push(c),
        Key::Ctrl('q') => return true,
        Key::Ctrl('o') => editor.start_open(),
        Key::Ctrl('s') => editor.start_save(),
        Key::Backspace => editor.buffer().backspace(),
        Key::Delete => editor.buffer().delete(),
        Key::Esc => editor.exit_command(),
        Key::Left => editor.buffer().move_caret(0, -1),
        Key::Right => editor.buffer().move_caret(0, 1),
        Key::Up => editor.buffer().move_caret(-1, 0),
        Key::Down => editor.buffer().move_caret(1, 0),
        Key::Home => editor.buffer().move_caret(0, std::i32::MIN / 2),
        Key::End => editor.buffer().move_caret(0, std::i32::MAX / 2),
        _ => {}
    }
    false
}
