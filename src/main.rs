use anyhow::Result;
use std::io;

use crossterm::event::KeyEventKind;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::{DefaultTerminal, prelude::*};
use ratatui::{
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};

fn main() -> Result<()> {
    basalt::run()
}
