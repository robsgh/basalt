use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use directories::ProjectDirs;
use log::debug;
use log::error;
use log::info;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::List;
use ratatui::widgets::ListItem;
use ratatui::widgets::ListState;
use ratatui::widgets::Paragraph;
use ratatui::{DefaultTerminal, prelude::*};

/// State of the Basalt application
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub enum BasaltState {
    #[default]
    Init,
    Running,
    Exiting,
}

/// Data model used by the Basalt application
#[derive(Debug, Default, Clone)]
struct BasaltData {
    files: Vec<PathBuf>,
    list_state: ListState,
}

/// The main Basalt Application
#[derive(Debug, Default, Clone)]
pub struct BasaltApp {
    state: BasaltState,
    data: BasaltData,
    data_path: PathBuf,
}

impl BasaltApp {
    /// Create a new directory for the Basalt project data
    fn create_new_dir(&self, dir: &PathBuf) -> Result<()> {
        info!(
            "Creating basalt data directory at {}",
            dir.to_string_lossy()
        );
        fs::create_dir_all(dir).with_context(|| {
            format!(
                "failed to create basalt data directory at {}",
                dir.to_string_lossy()
            )
        })?;

        Ok(())
    }

    /// Load `self.files` from a directory
    fn load_from_dir(&self, dir: &PathBuf) -> Result<Vec<PathBuf>> {
        info!("Basalt data directory exists, reusing it");

        let files = fs::read_dir(dir)?
            .filter_map(|d| d.ok().map(|entry| entry.path()))
            .collect();

        Ok(files)
    }

    /// Begin the application
    ///
    /// The application will beign in the [`BasaltState::Init`] state which will initialize the
    /// project directories as needed. Then, the terminal will be configured for the TUI
    /// application and the app will enter [`BasaltState::Running`]. Upon exit, the state will
    /// change to [`BasaltState::Exiting`] and the program will restore the terminal session.
    pub fn run(&mut self) -> Result<()> {
        if self.state == BasaltState::Init {
            let dirs = ProjectDirs::from("com", "Schminfra", "Basalt")
                .context("failed to create project directory for basalt")?;
            self.data_path = dirs.data_dir().to_path_buf();

            // TODO: Instead of just creating a dir and putting files in it, we should group dirs
            // and consider each subdir as a single top-level note bundle
            match fs::exists(&self.data_path) {
                Ok(false) => self.create_new_dir(&self.data_path)?,
                Ok(true) => {
                    let files = self.load_from_dir(&self.data_path)?;
                    self.data.files = files;
                }
                Err(e) => bail!("failed to stat basalt data dir: {:?}", e),
            };
        }

        let mut terminal = ratatui::init();
        let res = self.tui_loop(&mut terminal);
        ratatui::restore();

        res
    }

    /// Continuously draw and update the screen while the TUI is running. This will loop
    /// forever as long as `self.state` is not equal to [`BasaltState::Exiting`].
    fn tui_loop(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        info!("Entering running state");
        self.state = BasaltState::Running;

        while self.state != BasaltState::Exiting {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }

        Ok(())
    }

    /// Organize the immediate mode TUI layout and present it to the screen
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

    /// Refresh the list of files shown in the TUI
    fn refresh_file_list(&mut self) {
        match self.load_from_dir(&self.data_path) {
            Ok(files) => {
                debug!("refreshed file list: {:?}", files);
                self.data.files = files;
            }
            Err(e) => {
                error!(
                    "failed to load files in directory {:?}: {:?}",
                    self.data_path.to_string_lossy(),
                    e
                );
            }
        };
    }

    /// Handle keyboard events from the terminal
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
            KeyCode::Char('r') => {
                self.refresh_file_list();
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, fs::File};

    use tempfile::tempdir;

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

    #[test]
    fn handle_file_update() -> Result<()> {
        let dir = tempdir()?;
        let mut files = vec!["1.md", "2.md", "3.md"];
        for f in &files {
            File::create(dir.path().join(f))?;
        }
        fs::create_dir(dir.path().join("folder1"))?;
        File::create(dir.path().join("folder1/1.md"))?;

        let mut app = BasaltApp::default();
        app.data_path = dir.path().to_path_buf();

        app.handle_key_event(KeyCode::Char('r').into());

        drop(dir);

        files.push("folder1");
        assert_eq!(
            app.data
                .files
                .into_iter()
                .map(|p| p.strip_prefix(&app.data_path).unwrap().to_owned())
                .map(|p| p.to_string_lossy().to_string())
                .collect::<HashSet<_>>(),
            files.into_iter().map(String::from).collect::<HashSet<_>>()
        );

        Ok(())
    }
}
