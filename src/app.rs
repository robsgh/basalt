use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::symbols::border;
use ratatui::widgets::Block;
use ratatui::widgets::List;
use ratatui::widgets::ListItem;
use ratatui::widgets::ListState;
use ratatui::widgets::Padding;
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
    files: Vec<&'static str>,
    list_state: ListState,
}

#[derive(Debug, Default, Clone)]
pub struct BasaltApp {
    state: BasaltState,
    data: BasaltData,
}

impl BasaltApp {
    pub fn run(&mut self) -> Result<()> {
        self.data.files = vec!["file 1", "file 2", "file 3"];

        let mut terminal = ratatui::init();
        let res = self.ui_loop(&mut terminal);
        ratatui::restore();

        res
    }

    fn ui_loop(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while self.state != BasaltState::Exiting {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let title = Line::from(" Basalt ".bold());

        let block = Block::bordered()
            .title(title.centered())
            .padding(Padding::uniform(1))
            .border_set(border::THICK);

        let files = List::new(self.data.files.iter().map(|s| ListItem::from(*s)))
            .block(block)
            .highlight_style(Style::new().bold().black().on_white());

        frame.render_stateful_widget(files, frame.area(), &mut self.data.list_state);
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
        app.data.files = vec!["1", "2", "3"];

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
