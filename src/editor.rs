use std::{
    env,
    time::{Duration, Instant},
};
use syntect::parsing::SyntaxSet;
use syntect::{easy::HighlightLines, highlighting::ThemeSet};
use syntect::{parsing::SyntaxReference, util::as_24_bit_terminal_escaped};
use termion::{color, event::Key};

use crate::Document;
use crate::Row;
use crate::Terminal;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const STATUS_BG_COLOR: color::Rgb = color::Rgb(239, 239, 239);
const STATUS_FG_COLOR: color::Rgb = color::Rgb(63, 63, 63);

#[derive(Default)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

pub struct HighLightManager<'a> {
    highlighter: HighlightLines<'a>,
    ts: &'a ThemeSet,
    syntax: &'a SyntaxReference,
}

impl<'a> HighLightManager<'a> {
    pub fn default(ps: &'a SyntaxSet, ts: &'a ThemeSet) -> Self {
        let syntax = ps.find_syntax_by_extension("rs").unwrap();
        let highlighter = HighlightLines::new(syntax, &ts.themes["base16-mocha.dark"]);

        Self {
            highlighter,
            ts,
            syntax,
        }
    }

    pub fn change_theme(&mut self, theme: &str) {
        self.highlighter = HighlightLines::new(self.syntax, &self.ts.themes[theme]);
    }
}

struct StatusMessage {
    text: String,
    time: Instant,
}
impl StatusMessage {
    fn from(message: String) -> Self {
        Self {
            time: Instant::now(),
            text: message,
        }
    }
}

pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
    cursor_position: Position,
    document: Document,
    offset: Position,
    ss: SyntaxSet,
    status_message: StatusMessage,
}

impl Editor {
    pub fn run(&mut self, h: &mut HighLightManager) {
        loop {
            if let Err(error) = self.refresh_screen(h) {
                die(error);
            }
            if self.should_quit {
                break;
            }
            if let Err(error) = self.process_keypress(h) {
                die(error);
            }
        }
    }

    fn refresh_screen(&mut self, h: &mut HighLightManager) -> Result<(), std::io::Error> {
        Terminal::cursor_hide();
        Terminal::cursor_position(&Position::default());

        if self.should_quit {
            Terminal::clear_screen();
            println!("Goodbye.\r");
        } else {
            self.draw_rows(h);
            self.draw_status_bar();
            self.draw_message_bar();

            Terminal::cursor_position(&Position {
                x: self.cursor_position.x.saturating_sub(self.offset.x),
                y: self.cursor_position.y.saturating_sub(self.offset.y),
            });
        }

        Terminal::cursor_show();
        Terminal::flush()
    }

    fn draw_status_bar(&self) {
        let mut status;
        let width = self.terminal.size().width as usize;
        let mut file_name = "[No Name]".to_string();
        if let Some(name) = &self.document.file_name {
            file_name = name.clone();
            file_name.truncate(20);
        }
        status = format!("{} - {} lines", file_name, self.document.len());
        let line_indicator = format!(
            "{}/{} ",
            self.cursor_position.y.saturating_add(1),
            self.document.len()
        );
        let len = status.len() + line_indicator.len();

        if width > len {
            status.push_str(&" ".repeat(width - len));
        }
        status = format!("{}{}", status, line_indicator);
        status.truncate(width);

        Terminal::set_bg_color(STATUS_BG_COLOR);
        Terminal::set_fg_color(STATUS_FG_COLOR);
        println!("{}\r", status);
        Terminal::reset_fg_color();
        Terminal::reset_bg_color();
    }

    fn draw_message_bar(&self) {
        Terminal::clear_current_line();
        let message = &self.status_message;
        if Instant::now() - message.time < Duration::new(5, 0) {
            let mut text = message.text.clone();
            text.truncate(self.terminal.size().width as usize);
            print!("{}", text);
        }
    }

    fn process_keypress(&mut self, h: &mut HighLightManager) -> Result<(), std::io::Error> {
        let pressed_key = Terminal::read_key()?;

        match pressed_key {
            Key::Ctrl('q') => self.should_quit = true,
            Key::Ctrl('t') => h.change_theme("InspiredGitHub"),
            Key::Up
            | Key::Down
            | Key::Left
            | Key::Right
            | Key::PageUp
            | Key::PageDown
            | Key::Home
            | Key::End => self.move_cursor(pressed_key),
            _ => (),
        }

        self.scroll();

        Ok(())
    }

    fn scroll(&mut self) {
        let Position { x, y } = self.cursor_position;
        let width = self.terminal.size().width as usize;
        let height = self.terminal.size().height as usize;

        if x < self.offset.x {
            self.offset.x = x;
        } else if x >= self.offset.x.saturating_add(width) {
            self.offset.x = x.saturating_sub(width).saturating_add(1);
        }

        if y < self.offset.y {
            self.offset.y = y;
        } else if y >= self.offset.y.saturating_add(height) {
            self.offset.y = y.saturating_sub(height).saturating_add(1);
        }
    }

    fn move_cursor(&mut self, key: Key) {
        let Position { mut y, mut x } = self.cursor_position;

        let mut width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };
        let height = self.document.len();

        match key {
            Key::Up => y = y.saturating_sub(1),
            Key::Down => {
                if y < height {
                    y = y.saturating_add(1);
                }
            }
            Key::Left => x = x.saturating_sub(1),
            Key::Right => {
                if x < width {
                    x = x.saturating_add(1);
                }
            }
            // TODO: move page by page here and not to the beginning
            Key::PageUp => y = 0,
            Key::PageDown => y = height,
            Key::Home => x = 0,
            Key::End => x = width,
            _ => (),
        }

        width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };

        if x > width {
            x = width;
        }

        self.cursor_position = Position { x, y }
    }

    fn draw_welcome_message(&self) {
        let mut welcome_message = format!("Hector editor -- version {}\r", VERSION);

        let width = self.terminal.size().width as usize;
        let len = welcome_message.len();
        let padding = width.saturating_sub(len) / 2;
        let spaces = " ".repeat(padding.saturating_sub(1));

        welcome_message = format!("~{}{}", spaces, welcome_message);
        welcome_message.truncate(width);

        println!("{}\r", welcome_message);
    }

    pub fn draw_row(&self, row: &Row, h: &mut HighLightManager) {
        let width = self.terminal.size().width as usize;
        let start = self.offset.x;
        let end = self.offset.x + width;
        let row = row.render(start, end);
        let ranges = h.highlighter.highlight(&row, &self.ss);
        let escaped = as_24_bit_terminal_escaped(&ranges[..], true);

        println!("{}\r", escaped)
    }

    fn draw_rows(&self, h: &mut HighLightManager) {
        let height = self.terminal.size().height;

        for terminal_row in 0..height {
            Terminal::clear_current_line();
            if let Some(row) = self.document.row(terminal_row as usize + self.offset.y) {
                self.draw_row(row, h);
            } else if self.document.is_empty() && terminal_row == height / 3 {
                self.draw_welcome_message();
            } else {
                println!("~\r");
            }
        }
    }

    pub fn default(args: Vec<String>) -> Self {
        let ps = SyntaxSet::load_defaults_newlines();
        let mut initial_status = String::from("HELP: Ctrl-Q = quit");

        let document = if args.len() > 1 {
            let file_name = &args[1];
            let doc = Document::open(&file_name);
            if doc.is_ok() {
                doc.unwrap()
            } else {
                initial_status = format!("ERR: Could not open file: {}", file_name);
                Document::default()
            }
        } else {
            Document::default()
        };

        Self {
            should_quit: false,
            terminal: Terminal::default().expect("Failed to initialize terminal"),
            cursor_position: Position { x: 0, y: 0 },
            document,
            offset: Position::default(),
            ss: ps,
            status_message: StatusMessage::from(initial_status),
        }
    }
}

fn die(e: std::io::Error) {
    print!("{}", termion::clear::All);
    panic!("{}", e);
}
