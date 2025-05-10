use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Paragraph, Widget},
};

pub enum Show {
    Visible,
    Hidden,
}

pub struct Console {
    input: String,
    pub character_index: usize,
    pub display: Show,
}

impl Console {
    pub fn new() -> Self {
        Self {
            input: String::from(":"),
            character_index: 1,
            display: Show::Hidden,
        }
    }

    pub fn show_console(&mut self) {
        self.display = Show::Visible;
    }

    pub fn hide_console(&mut self) {
        self.display = Show::Hidden;
        self.character_index = 1;
        self.display = Show::Hidden;
        self.input = String::from(":");
    }

    pub fn set_prefix(&mut self, keyword: &str){
        self.input = String::from(keyword);
        self.character_index = self.input.len();
        self.display = Show::Visible;
    }

    fn cursor_left(&mut self) {
        let left_move = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(left_move)
    }

    fn cursor_right(&mut self) {
        let right_move = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(right_move);
    }

    fn clamp_cursor(&self, new_pos: usize) -> usize {
        new_pos.clamp(0, self.input.chars().count())
    }

    pub fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.cursor_right();
    }

    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    pub fn delete_char(&mut self) {
        if self.character_index != 0 {
            let before_delete = self.input.chars().take(self.character_index - 1);
            let after_delete = self.input.chars().skip(self.character_index);
            self.input = before_delete.chain(after_delete).collect();
            self.cursor_left();
        }
    }

    pub fn submit_command(&mut self) -> String {
        self.character_index = 1;
        self.display = Show::Hidden;
        let result = self.input.clone();
        self.input = String::from(":");
        result
    }
}

impl Widget for &mut Console {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.display {
            Show::Visible => {
                Paragraph::new(self.input.as_str()).style(Style::default()).render(area, buf);
            },
            Show::Hidden => {}
        }
    }
}
