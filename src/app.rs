use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::symbols::border;
use ratatui::widgets::Block;
use ratatui::widgets::List;
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
pub struct BasaltApp {
    state: BasaltState,
}

impl BasaltApp {
    pub fn run(&mut self) -> Result<()> {
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

        let files = List::new(
            // self.files
            //     .iter()
            //     .map(|s| ListItem::new(*s))
            //     .collect::<Vec<ListItem>>(),
            ["file 1", "file 2", "file 3"],
        )
        .block(block)
        .highlight_style(Style::new().bold().black().on_white());

        // frame.render_stateful_widget(files, frame.area(), &mut self.file_list_state);
        frame.render_widget(files, frame.area());
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
            // KeyCode::Char('j') | KeyCode::Down => {
            //     if self.file_list_state.selected().is_none() {
            //         self.file_list_state.select_first();
            //     } else {
            //         self.file_list_state.select_next();
            //     }
            // }
            // KeyCode::Char('k') | KeyCode::Up => {
            //     if self.file_list_state.selected().is_none() {
            //         self.file_list_state.select_last();
            //     } else {
            //         self.file_list_state.select_previous();
            //     }
            // }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render() {
        // let interface = BasaltInterface::default();
        // let mut buf = Buffer::empty(Rect::new(0, 0, 50, 4));

        // interface.render(buf.area, &mut buf);

        // let title_style = Style::new().bold();
        // let counter_style = Style::new().yellow();
        // let key_style = Style::new().blue().bold();

        // let mut snapshot = Buffer::with_lines(vec![
        //     "┏━━━━━━━━━━━━━ Counter App Tutorial ━━━━━━━━━━━━━┓",
        //     "┃                    Value: 0                    ┃",
        //     "┃                                                ┃",
        //     "┗━ Decrement <Left> Increment <Right> Quit <Q> ━━┛",
        // ]);
        // snapshot.set_style(Rect::new(14, 0, 22, 1), title_style);
        // snapshot.set_style(Rect::new(28, 1, 1, 1), counter_style);
        // snapshot.set_style(Rect::new(13, 3, 6, 1), key_style);
        // snapshot.set_style(Rect::new(30, 3, 7, 1), key_style);
        // snapshot.set_style(Rect::new(43, 3, 4, 1), key_style);

        // assert_eq!(buf, snapshot);
    }

    #[test]
    fn test_handle_key_event() -> Result<()> {
        let mut app = BasaltApp::default();

        app.handle_key_event(KeyCode::Char('q').into());
        assert_eq!(app.state, BasaltState::Exiting);

        Ok(())
    }
}
