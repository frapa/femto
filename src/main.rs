use std::io::{stdin, stdout, Stdout, Write};
use termion::{cursor::Goto, event::Key, input::TermRead, raw::IntoRawMode};

#[derive(Default)]
struct Editor {
    buffer: Buffer,
}

#[derive(Default)]
struct Buffer {
    row: u16,
    col: u16,
    lines: Vec<String>,
}

impl Buffer {
    fn push(&mut self, c: char) {
        self.lines.get_mut(self.row as usize).unwrap().push(c);
        self.col += 1;
    }
}

fn main() {
    let mut stdout = stdout().into_raw_mode().expect("Unsupported terminal.");
    let mut editor = Editor::default();
    editor.buffer.lines.push(String::new());

    loop {
        print_screen(&mut stdout, &editor);
        if handle_keys(&mut editor) {
            break;
        }
    }
}

fn print_screen(stdout: &mut Stdout, editor: &Editor) {
    let buffer = &editor.buffer;
    let (w, h) = termion::terminal_size().expect("Unsupported terminal.");

    write!(stdout, "{}{}", termion::clear::All, Goto(1, 1)).unwrap();
    for line in buffer.lines.iter() {
        write!(stdout, "{}\n\r", line).unwrap();
    }
    for _ in 0..(h - 1 - buffer.lines.len() as u16) {
        write!(stdout, "~\n\r").unwrap();
    }
    write!(stdout, "{}", Goto(buffer.col + 1, buffer.row + 1)).unwrap();
    stdout.flush().unwrap();
}

fn handle_keys(editor: &mut Editor) -> bool {
    let c = stdin().keys().next().unwrap();
    match c.unwrap() {
        Key::Char(c) => editor.buffer.push(c),
        Key::Ctrl('q') => return true,
        _ => {}
    }
    false
}
