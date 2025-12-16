use std::fs;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use anyhow::bail;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use directories::ProjectDirs;
use log::info;
use ratatui::symbols::border;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::List;
use ratatui::widgets::ListItem;
use ratatui::widgets::ListState;
use ratatui::widgets::Padding;
use ratatui::widgets::Paragraph;
use ratatui::{DefaultTerminal, prelude::*};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub enum BasaltState {
    #[default]
    Init,
    Running,
    Exiting,
}

#[derive(Debug, Default, Clone)]
struct BasaltData {
    files: Vec<PathBuf>,
    list_state: ListState,
}

#[derive(Debug, Default, Clone)]
pub struct BasaltApp {
    state: BasaltState,
    data: BasaltData,
    data_path: PathBuf,
}

impl BasaltApp {
    fn create_new_dirs(&self, dirs: &ProjectDirs) -> Result<()> {
        info!(
            "Creating basalt data directory at {}",
            dirs.data_dir().to_string_lossy()
        );
        fs::create_dir_all(dirs.data_dir()).with_context(|| {
            format!(
                "failed to create basalt data directory at {}",
                dirs.data_dir().to_string_lossy()
            )
        })?;

        Ok(())
    }

    fn load_from_dirs(&self, dirs: &ProjectDirs) -> Result<Vec<PathBuf>> {
        info!("Basalt data directory exists, reusing it");

        let files = fs::read_dir(dirs.data_dir())?
            .filter_map(|d| d.ok().map(|entry| entry.path()))
            .collect();

        Ok(files)
    }

    pub fn run(&mut self) -> Result<()> {
        if self.state == BasaltState::Init {
            let dirs = ProjectDirs::from("com", "Schminfra", "Basalt")
                .context("failed to create project directory for basalt")?;

            match fs::exists(dirs.data_dir()) {
                Ok(false) => self.create_new_dirs(&dirs)?,
                Ok(true) => {
                    let files = self.load_from_dirs(&dirs)?;
                    self.data.files = files;
                }
                Err(e) => bail!("failed to stat basalt data dir: {:?}", e),
            };

            self.data_path = dirs.data_dir().to_owned();
        }

        let mut terminal = ratatui::init();
        let res = self.tui_loop(&mut terminal);
        ratatui::restore();

        res
    }

    fn tui_loop(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while self.state != BasaltState::Exiting {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let [left_nav_area, main_area, right_bar_area] = Layout::horizontal([
            Constraint::Percentage(10),
            Constraint::Percentage(80),
            Constraint::Percentage(10),
        ])
        .areas::<3>(frame.area());

        let left_nav_block = Block::new()
            .borders(Borders::LEFT | Borders::TOP | Borders::BOTTOM)
            .title_alignment(Alignment::Center)
            .title_style(Style::new().bold())
            .title("Left Nav");
        let main_block = Block::bordered()
            .title_alignment(Alignment::Center)
            .title("Basalt");
        let right_block = Block::new()
            .borders(Borders::RIGHT | Borders::TOP | Borders::BOTTOM)
            .title_alignment(Alignment::Center)
            .title_style(Style::new().bold())
            .title("Todo");

        let files = List::new(
            self.data
                .files
                .iter()
                .filter_map(|pb| {
                    pb.strip_prefix(&self.data_path)
                        .ok()
                        .map(|p| p.to_string_lossy())
                })
                .map(|f| ListItem::from(f)),
        )
        .block(left_nav_block)
        .highlight_style(Style::new().bold().black().on_white());
        frame.render_stateful_widget(files, left_nav_area, &mut self.data.list_state);

        let main = Paragraph::new("This is the main content. There are many like it, but this one is hastily created and mine.").centered().block(main_block);
        frame.render_widget(main, main_area);

        let right_bar = List::new(vec![
            ListItem::from("[ ] Todo 1"),
            ListItem::from("[ ] Todo 2"),
            ListItem::from("[ ] Todo 3"),
        ])
        .block(right_block);
        frame.render_widget(right_bar, right_bar_area);
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.state = BasaltState::Exiting;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if self.data.list_state.selected().is_none() {
                    self.data.list_state.select_first();
                } else {
                    self.data.list_state.select_next();
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.data.list_state.selected().is_none() {
                    self.data.list_state.select_last();
                } else {
                    self.data.list_state.select_previous();
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_key_event() -> Result<()> {
        let mut app = BasaltApp::default();

        app.handle_key_event(KeyCode::Char('q').into());
        assert_eq!(app.state, BasaltState::Exiting);

        Ok(())
    }

    #[test]
    fn test_file_list_nav() -> Result<()> {
        let mut app = BasaltApp::default();
        app.data.files = vec!["1", "2", "3"].into_iter().map(PathBuf::from).collect();

        assert_eq!(app.data.list_state.selected(), None);

        // up down nav
        app.handle_key_event(KeyCode::Char('j').into());
        assert_eq!(app.data.list_state.selected(), Some(0));
        app.handle_key_event(KeyCode::Char('j').into());
        assert_eq!(app.data.list_state.selected(), Some(1));
        app.handle_key_event(KeyCode::Char('k').into());
        assert_eq!(app.data.list_state.selected(), Some(0));

        // cannot go up beyond top
        app.handle_key_event(KeyCode::Char('k').into());
        assert_eq!(app.data.list_state.selected(), Some(0));

        // cannot go down beyond bottom
        app.data.list_state.select_last();
        let last_idx = app.data.list_state.selected();
        app.handle_key_event(KeyCode::Char('j').into());
        assert_eq!(app.data.list_state.selected(), last_idx);

        Ok(())
    }
}
