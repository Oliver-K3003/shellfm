use crossterm::event::{KeyEventKind, KeyCode};
use ratatui::{layout::Position, style::Style, widgets::Paragraph};
use tokio::sync::mpsc::UnboundedSender;
use color_eyre::Result;

use crate::action::Action;

use super::Component;


enum Show{
    Visible,
    Hidden
}

pub struct Console {
    input: String,
    character_index: usize,
    display: Show,
    command_tx: Option<UnboundedSender<Action>>,
}

impl Console{
    pub fn new() -> Self {
        Self {
            input: String::new(),
            character_index: 1,
            display: Show::Hidden,
            command_tx: None,
        }
    }

    fn show_console(&mut self){
        self.display = Show::Visible;
    }

    fn hide_reset(&mut self){
        self.display = Show::Hidden;
        self.character_index = 1;
        self.input = String::new();
    }

    fn clamp_cursor(&mut self, new_pos: usize) -> usize {
        new_pos.clamp(0, self.input.chars().count())
    }

    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    fn cursor_right(&mut self){
        let right_move = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(right_move);
    }

    fn cursor_left(&mut self){
        let left_move = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(left_move);
    }
    
    fn enter_char(&mut self, new_char: char){
        let idx = self.byte_index();
        self.input.insert(idx, new_char);
        self.cursor_right();
    }

    fn delete_char(&mut self) {
        if self.character_index != 0 {
            let before_delete = self.input.chars().take(self.character_index -1);
            let after_delete = self.input.chars().skip(self.character_index);
            self.input = before_delete.chain(after_delete).collect();
            self.cursor_left();
        }
    }

    fn submit_command(&mut self) -> String {
        self.character_index = 1;
        self.display = Show::Hidden;
        let result = self.input.clone();
        self.input = String::new();
        result
    }

    fn handle_cmd(&mut self, cmd: String) -> Result<Option<Action>>{
        match &cmd[..]{
            ":q" => Ok(Some(Action::Quit)),
            _ => Ok(Some(Action::DirMode)),
        }
    }
}

impl Component for Console {
    fn register_action_handler(&mut self, tx: tokio::sync::mpsc::UnboundedSender<crate::action::Action>) -> color_eyre::eyre::Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> Result<Option<Action>> {
        if key.kind != KeyEventKind::Press {
            return Ok(None);
        }
        match key.code{
            KeyCode::Enter => {
                // Handle Commands
                let cmd = self.submit_command();
                self.handle_cmd(cmd)
            },
            KeyCode::Backspace => {
                self.delete_char();
                Ok(None)
            },
            KeyCode::Esc => {
                self.hide_reset();
                Ok(Some(Action::DirMode))
            },
            KeyCode::Char(x) => {
                self.enter_char(x);
                Ok(None)
            },
            _ => Ok(None),
        }
    }

    fn update(&mut self, action: crate::action::Action) -> Result<Option<Action>> {
        match action{
            Action::Tick => {}
            Action::Render => {}
            Action::ShowConsole => self.show_console(),
            _ => {},
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut ratatui::Frame, area: ratatui::prelude::Rect) -> color_eyre::eyre::Result<()> {
        match self.display {
            Show::Visible => {
                frame.render_widget(Paragraph::new(self.input.as_str()).style(Style::default()), area);
                frame.set_cursor_position(Position::new(
                        area.x + self.character_index as u16,
                        area.y,
                ));
            },
            Show::Hidden => {}
        }
        Ok(())
    }
}
