#![warn(clippy::all, clippy::pedantic)]
mod document;
mod editor;
mod row;
mod terminal;

use std::env;
use syntect::{highlighting::ThemeSet, parsing::SyntaxSet};

pub use document::Document;
pub use editor::Editor;
pub use editor::HighLightManager;
pub use editor::Position;
pub use row::Row;
pub use terminal::Terminal;

fn main() {
    let ts = ThemeSet::load_defaults();
    let ps = SyntaxSet::load_defaults_newlines();
    let mut hm = HighLightManager::default(&ps, &ts);

    Editor::default(env::args().collect()).run(&mut hm);
}
