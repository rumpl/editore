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
use syntect::{easy::HighlightLines, highlighting::ThemeSet, parsing::SyntaxSet};
pub use terminal::Terminal;

fn main() {
    let ts = ThemeSet::load_defaults();
    let ps = SyntaxSet::load_defaults_newlines();
    let syntax = ps.find_syntax_by_extension("rs").unwrap();
    let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);

    Editor::default(env::args().collect()).run(&mut h);
}
