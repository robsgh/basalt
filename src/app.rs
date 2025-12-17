use std::fs;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use directories::ProjectDirs;
use directories::UserDirs;
use log::info;
use ratatui::widgets::Block;
use ratatui::widgets::List;
use ratatui::widgets::ListItem;
use ratatui::widgets::ListState;
use ratatui::widgets::Padding;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Wrap;
use ratatui::{DefaultTerminal, prelude::*};

use crate::bundles::Bundle;
use crate::bundles::BundleLoader;

/// State of the Basalt application
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub enum BasaltState {
    #[default]
    Init,
    Running,
    Exiting,
}

/// The main Basalt Application
#[derive(Debug, Default, Clone)]
pub struct BasaltApp {
    /// Running state of Basalt
    state: BasaltState,
    bundle: Bundle,
    list_state: ListState,
}

impl BasaltApp {
    /// Get the project-only dirs for the application
    ///
    /// This is for Basalt config, state, and application data
    fn project_dirs() -> Result<ProjectDirs> {
        ProjectDirs::from("com", "Schminfra", "Basalt")
            .context("failed to create project directory for basalt")
    }

    /// Begin the application
    ///
    /// The application will beign in the [`BasaltState::Init`] state which will initialize the
    /// project directories as needed. Then, the terminal will be configured for the TUI
    /// application and the app will enter [`BasaltState::Running`]. Upon exit, the state will
    /// change to [`BasaltState::Exiting`] and the program will restore the terminal session.
    pub fn run(&mut self) -> Result<()> {
        if self.state == BasaltState::Init {
            if let Some(dirs) = UserDirs::new()
                && let Some(docs) = dirs.document_dir()
            {
                let default_bundle = docs.join("init");

                if fs::exists(&default_bundle).context("failed to check dir existence of Bundle")? {
                    self.bundle = BundleLoader::new(&default_bundle).load()?;
                } else {
                    self.bundle = BundleLoader::new(&default_bundle).init()?;
                }
            } else {
                bail!("Failed to load ~/Documents directory from UserDirs")
            }
        }

        info!("Entering running state");
        self.state = BasaltState::Running;

        let mut terminal = ratatui::init();
        let res = self.tui_loop(&mut terminal);
        ratatui::restore();

        res
    }

    /// Continuously draw and update the screen while the TUI is running. This will loop
    /// forever as long as `self.state` is not equal to [`BasaltState::Exiting`].
    fn tui_loop(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while self.state != BasaltState::Exiting {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }

        Ok(())
    }

    /// Organize the immediate mode TUI layout and present it to the screen
    fn draw(&mut self, frame: &mut Frame) {
        let [title_chunk, content_chunk] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(frame.area());
        let [left_chunk, body_chunk] =
            Layout::horizontal([Constraint::Max(30), Constraint::Fill(1)]).areas(content_chunk);
        let [nav_chunk, todo_chunk] =
            Layout::vertical([Constraint::Percentage(75), Constraint::Percentage(25)])
                .areas(left_chunk);

        let nav_block = Block::bordered()
            .title_alignment(Alignment::Center)
            .title_style(Style::new().bold())
            .title("Navigation");
        let todo_block = Block::bordered()
            .title_alignment(Alignment::Center)
            .title_style(Style::new().bold())
            .title("Todo");
        let mut main_block = Block::bordered().title_alignment(Alignment::Center);

        let notes = List::new(self.bundle.get_note_names())
            .block(nav_block)
            .highlight_style(Style::new().bold().black().on_white());
        frame.render_stateful_widget(notes, nav_chunk, &mut self.list_state);

        let title_text = Text::from(self.bundle.name()).centered().reversed();
        frame.render_widget(title_text, title_chunk);

        let main_contents = match &self.list_state.selected() {
            None => "No file selected.",
            Some(idx) => {
                if let Some(n) = self.bundle.get_note(*idx) {
                    main_block = main_block.title_top(n.name());
                    n.get().unwrap_or("<<error loading file>>")
                } else {
                    "<<file index not found?>>"
                }
            }
        };
        let body = Paragraph::new(main_contents)
            .block(main_block)
            .wrap(Wrap::default());
        frame.render_widget(body, body_chunk);

        let right_bar = List::new(vec![
            ListItem::from("[ ] Todo 1"),
            ListItem::from("[ ] Todo 2"),
            ListItem::from("[ ] Todo 3"),
        ])
        .block(todo_block);
        frame.render_widget(right_bar, todo_chunk);
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)?
            }
            _ => {}
        }

        Ok(())
    }

    /// Select the next file in the file list and update the data model's contents to ensure that
    /// the file is loaded on next render
    fn select_next_file(&mut self) {
        if self.list_state.selected().is_none() {
            self.list_state.select_first();
        } else {
            self.list_state.select_next();
        }
    }

    /// Select the previous file in the file list and update the data model's contents to ensure
    /// that the file is loaded on next render
    fn select_prev_file(&mut self) {
        if self.list_state.selected().is_none() {
            self.list_state.select_last();
        } else {
            self.list_state.select_previous();
        }
    }

    /// Handle keyboard events from the terminal
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.state = BasaltState::Exiting;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.select_next_file();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.select_prev_file();
            }
            KeyCode::Char('r') => {
                self.bundle = BundleLoader::new(self.bundle.get_path()).load()?;
            }
            _ => {}
        }

        Ok(())
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

        app.handle_key_event(KeyCode::Char('q').into())?;
        assert_eq!(app.state, BasaltState::Exiting);

        Ok(())
    }

    #[test]
    fn test_file_list_nav() -> Result<()> {
        let mut app = BasaltApp::default();
        assert_eq!(app.list_state.selected(), None);

        // mock out the bundle
        let dir = tempdir()?;
        let path = dir.path().join("test_file_list_nav");
        let bundle = BundleLoader::new(&path).init()?;
        File::create_new(path.join("1.md"))?;
        File::create_new(path.join("2.md"))?;
        app.bundle = bundle;

        // up down nav
        app.handle_key_event(KeyCode::Char('j').into())?;
        assert_eq!(app.list_state.selected(), Some(0));
        app.handle_key_event(KeyCode::Char('j').into())?;
        assert_eq!(app.list_state.selected(), Some(1));
        app.handle_key_event(KeyCode::Char('k').into())?;
        assert_eq!(app.list_state.selected(), Some(0));

        // cannot go up beyond top
        app.handle_key_event(KeyCode::Char('k').into())?;
        assert_eq!(app.list_state.selected(), Some(0));

        // cannot go down beyond bottom
        app.list_state.select_last();
        let last_idx = app.list_state.selected();
        app.handle_key_event(KeyCode::Char('j').into())?;
        assert_eq!(app.list_state.selected(), last_idx);

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
        let bundle = BundleLoader::new(dir.path()).load()?;
        app.bundle = bundle;

        app.handle_key_event(KeyCode::Char('r').into())?;

        drop(dir);

        files.push("folder1");
        assert_eq!(
            app.bundle
                .get_note_names()
                .into_iter()
                .collect::<HashSet<_>>(),
            files.into_iter().map(String::from).collect::<HashSet<_>>()
        );

        Ok(())
    }
}
