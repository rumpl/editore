#![warn(clippy::all, clippy::pedantic)]
mod document;
mod editor;
mod row;
mod terminal;

use std::env;

pub use document::Document;
pub use editor::Editor;
pub use editor::Position;
pub use row::Row;
use syntect::highlighting::ThemeSet;
pub use terminal::Terminal;

fn main() {
    let ts = ThemeSet::load_defaults();

    Editor::default(env::args().collect(), &ts).run();
}
