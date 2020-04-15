use std::cmp::{max, min};
use std::fs::File;
use std::io::{prelude::*, stdin, stdout, BufReader, Stdout, Write};
use std::path::PathBuf;
use termion::{cursor::Goto, event::Key, input::TermRead, raw::IntoRawMode};

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
    fn move_caret(&mut self, row: i16, col: i16);
}

struct Editor {
    file_buffer: FileBuffer,
    command: State,
    message: Option<String>,
}

impl Editor {
    fn new() -> Self {
        Self {
            file_buffer: FileBuffer::new(),
            command: State::Femto,
            message: None,
        }
    }

    fn buffer(&mut self) -> &mut dyn Buffer {
        match &mut self.command {
            State::Femto => &mut self.file_buffer,
            State::Cmd((_, buffer)) => buffer,
        }
    }

    fn push(&mut self, c: char) {
        if c == '\n' {
            match &self.command {
                State::Femto => self.buffer().push(c),
                State::Cmd((command, buffer)) => match command {
                    Command::Open => self.open(PathBuf::from(&buffer.line)),
                    Command::Save => self.save(PathBuf::from(&buffer.line)),
                },
            }
        } else {
            self.buffer().push(c);
        }
    }

    fn start_open(&mut self) {
        self.command = State::Cmd((Command::Open, LineBuffer::default()));
    }

    fn open(&mut self, path: PathBuf) {
        match self.file_buffer.load(path.clone()) {
            Ok(_) => self.exit_command(),
            Err(err) => self.show_message(err.to_string()),
        }
    }

    fn start_save(&mut self) {
        let mut buffer = LineBuffer::default();
        buffer.line = self.file_buffer.path.to_str().unwrap().to_string();
        buffer.col = buffer.line.len() as u16;
        self.command = State::Cmd((Command::Save, buffer));
    }

    fn save(&mut self, path: PathBuf) {
        match self.file_buffer.save(path.clone()) {
            Ok(_) => self.exit_command(),
            Err(err) => self.show_message(err.to_string()),
        }
    }

    fn prompt(&self) -> (String, u16, u16) {
        match &self.command {
            State::Femto => match &self.message {
                Some(message) => (message.clone(), 0, 0),
                None => (String::from("femto"), 0, 0),
            },
            State::Cmd((command, buffer)) => match command {
                Command::Open => (format!("Open file at: {}", buffer.line), 15, buffer.col),
                Command::Save => (format!("Save file at: {}", buffer.line), 15, buffer.col),
            },
        }
    }

    fn show_message(&mut self, message: String) {
        self.exit_command();
        self.message = Some(message);
    }

    fn exit_command(&mut self) {
        self.command = State::Femto;
    }
}

struct FileBuffer {
    row: u16,
    col: u16,
    path: PathBuf,
    lines: Vec<String>,
}

impl FileBuffer {
    fn new() -> Self {
        Self {
            row: 0,
            col: 0,
            path: PathBuf::default(),
            lines: vec![String::new()],
        }
    }

    fn line(&mut self) -> &mut String {
        self.lines.get_mut(self.row as usize).unwrap()
    }

    fn load(&mut self, path: PathBuf) -> Result<(), std::io::Error> {
        let file = File::open(path.clone())?;
        self.lines = BufReader::new(file).lines().map(|l| l.unwrap()).collect();
        self.path = path;
        self.row = 0;
        self.col = 0;
        Ok(())
    }

    fn save(&self, path: PathBuf) -> Result<(), std::io::Error> {
        let mut file = File::create(path.clone())?;
        for line in self.lines.iter() {
            writeln!(file, "{}", line).unwrap();
        }
        Ok(())
    }
}

impl Buffer for FileBuffer {
    fn push(&mut self, c: char) {
        if c == '\n' {
            let row = self.row as usize;
            self.lines.insert(row + 1, String::new());
            self.col = 0;
            self.row += 1;
            return;
        }

        let col = self.col as usize;
        self.line().insert(col, c);
        self.col += 1;
    }

    fn backspace(&mut self) {
        let col = self.col as usize;
        let row = self.row as usize;

        if self.col == 0 && row != 0 {
            self.lines.remove(row);
            self.row -= 1;
            self.col = self.line().len() as u16;
        } else if col != 0 {
            self.line().remove(col - 1);
            self.col -= 1;
        }
    }

    fn move_caret(&mut self, row: i16, col: i16) {
        let num_lines = self.lines.len() as i16;
        self.row = min(max(self.row as i16 + row, 0), num_lines - 1) as u16;
        let line_len = self.line().len() as i16;
        self.col = min(max(self.col as i16 + col, 0), line_len) as u16;
    }
}

#[derive(Default)]
struct LineBuffer {
    col: u16,
    line: String,
}

impl Buffer for LineBuffer {
    fn push(&mut self, c: char) {
        if c != '\n' {
            self.line.insert(self.col as usize, c);
            self.col += 1;
        }
    }

    fn backspace(&mut self) {
        if self.col != 0 {
            self.line.remove(self.col as usize - 1);
            self.col -= 1;
        }
    }

    fn move_caret(&mut self, _: i16, col: i16) {
        let line_len = self.line.len() as i16;
        self.col = min(max(self.col as i16 + col, 0), line_len) as u16;
    }
}

fn main() {
    let mut stdout = stdout().into_raw_mode().expect("Unsupported terminal.");
    let mut editor = Editor::new();

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
    let file_buffer = &editor.file_buffer;
    let (row, col) = (file_buffer.row + 1, file_buffer.col + 1);
    let (w, h) = termion::terminal_size().expect("Unsupported terminal.");

    // Clear and start writing from origin
    write!(stdout, "{}{}", termion::clear::All, Goto(1, 1)).unwrap();

    // Content
    for line in file_buffer.lines.iter() {
        write!(stdout, "{}\n\r", line).unwrap();
    }

    // ~ as filler for parts of the window that are outside the buffer
    for _ in 0..(h - 1 - file_buffer.lines.len() as u16) {
        write!(stdout, "~\n\r").unwrap();
    }

    // Status bar
    let (prompt, prompt_len, prompt_col) = editor.prompt();
    // 12 for the text ("row:", etc), 10 is extra padding space for the values
    let spacer = " ".repeat(w as usize - prompt.len() - 12 - 10);
    write!(stdout, "{}{}row: {}, col: {}", prompt, spacer, row, col).unwrap();

    // Draw cursor on the right place
    match editor.command {
        State::Femto => write!(stdout, "{}", Goto(col, row)).unwrap(),
        _ => write!(stdout, "{}", Goto(prompt_len + prompt_col, h)).unwrap(),
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
        Key::Left => editor.buffer().move_caret(0, -1),
        Key::Right => editor.buffer().move_caret(0, 1),
        Key::Up => editor.buffer().move_caret(-1, 0),
        Key::Down => editor.buffer().move_caret(1, 0),
        _ => {}
    }
    false
}
