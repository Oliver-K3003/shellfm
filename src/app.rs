use ratatui::{
    DefaultTerminal,
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Position, Rect},
    style::{Color, Style, Styled, Stylize, palette::tailwind},
    symbols,
    text::Line,
    widgets::{
        Block, Borders, HighlightSpacing, List, ListItem, Paragraph, StatefulWidget, Widget,
    },
};
use std::{
    env, error, fmt::{self, Display}, fs, path::Path
};

pub mod console;
pub mod display;
use console::{Console, Show};
use display::{Dir, DirList};

// set global styles
const HEADER_STYLE: Style = Style::new()
    .fg(tailwind::ZINC.c300)
    .bg(tailwind::SLATE.c700);
const ROW_BG: Color = tailwind::SLATE.c800;
const ALT_ROW_BG: Color = tailwind::SLATE.c900;
const SELECTED_STYLE: Style = Style::new().bg(tailwind::SLATE.c600);
const TEXT_COLOR: Color = tailwind::ZINC.c300;

// ----- MODE ----- //
pub enum Mode {
    Normal,
    Visual,
    Insert,
    Command,
}

impl Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let val: &str = match self {
            Mode::Normal => "Normal",
            Mode::Insert => "Insert",
            Mode::Visual => "Visual",
            _ => "",
        };
        write!(f, "{}", val)
    }
}

// ----- STATE ----- //
pub enum State {
    Editing,
}

// ----- APP ----- //
pub struct App {
    pub mode: Mode,
    state: Option<State>,
    pub dir_list: DirList,
    pub should_exit: bool,
    pub curr_dir: String,
    pub console: Console,
    pub shellpos: Option<Rect>,
}

impl App {
    pub fn new() -> Self {
        let path = env::current_dir().unwrap();
        Self {
            mode: Mode::Normal,
            should_exit: false,
            state: None,
            dir_list: DirList::new(&path),
            curr_dir: path.display().to_string(),
            console: Console::new(),
            shellpos: None,
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<(), Box<dyn error::Error>> {
        self.dir_list.state.select_first();
        while !self.should_exit {
            terminal.draw(|frame| {
                frame.render_widget(&mut self, frame.area());
                match self.console.display {
                    Show::Visible => {
                        if let Some(rect) = self.shellpos {
                            frame.set_cursor_position(Position::new(
                                rect.x + self.console.character_index as u16,
                                rect.y + 1,
                            ));
                        }
                    }
                    Show::Hidden => {}
                }
            })?;
            if let Event::Key(key) = event::read()? {
                self.handle_key(key);
            };
        }
        Ok(())
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        match self.mode {
            Mode::Normal => {
                match key.code {
                    KeyCode::Char('j') | KeyCode::Down => self.select_next(),
                    KeyCode::Char('k') | KeyCode::Up => self.select_prev(),
                    KeyCode::Char('h') | KeyCode::Left => self.move_up_dir(),
                    KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => self.move_down_dir(),
                    KeyCode::Char('g') => {
                        if let Event::Key(key) = event::read().unwrap() {
                            match key.code {
                                KeyCode::Char('g') => self.select_first(),
                                _ => self.handle_key(key),
                            }
                        }
                    }
                    KeyCode::Home => self.select_first(),
                    KeyCode::Char('d') => {
                        if let Event::Key(key) = event::read().unwrap() {
                            match key.code {
                                KeyCode::Char('d') => self.delete_selected(),
                                _ => self.handle_key(key),
                            }
                        }
                    },
                    KeyCode::Char('G') | KeyCode::End => self.select_last(),
                    KeyCode::Char('i') => self.enter_insert('i'),
                    KeyCode::Char('a') => self.enter_insert('a'),
                    KeyCode::Char(':') => {
                        self.console.show_console();
                        self.mode = Mode::Command;
                    }
                    _ => {}
                };
            }

            Mode::Insert => {
                match key.code {
                    // make file
                    KeyCode::Char('f') => {
                        self.console.set_prefix("Enter File Name:");
                        self.mode = Mode::Command;
                    }
                    // make directory
                    KeyCode::Char('d') => {
                        self.console.set_prefix("Enter Directory Name:");
                        self.mode = Mode::Command;
                    }
                    KeyCode::Esc => self.mode = Mode::Normal,
                    _ => {}
                }
            }

            Mode::Command => {
                match key.code {
                    KeyCode::Esc => {
                        self.console.hide_console();
                        self.mode = Mode::Normal;
                    }
                    KeyCode::Enter => {
                        let val = &self.console.submit_command()[..];
                        self.mode = Mode::Normal;
                        // match val to various commands
                        self.handle_command(val);
                    }
                    KeyCode::Backspace => self.console.delete_char(),
                    KeyCode::Char(x) => self.console.enter_char(x),
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn enter_insert(&mut self, inp: char) {
        self.mode = Mode::Insert;
        if inp == 'i' {
            self.dir_list.state.select_first();
        } else {
            self.dir_list.state.select_last();
        }
    }

    fn select_next(&mut self) {
        self.dir_list.state.select_next();
    }

    fn select_prev(&mut self) {
        self.dir_list.state.select_previous();
    }

    fn select_first(&mut self) {
        self.dir_list.state.select_first();
    }

    fn select_last(&mut self) {
        self.dir_list.state.select_last();
    }

    fn move_up_dir(&mut self) {
        let parent = Path::new(&self.curr_dir).parent().unwrap_or(Path::new("/"));
        self.dir_list = DirList::new(parent);
        self.curr_dir = parent.display().to_string();
        self.dir_list.state.select_first();
    }

    fn move_down_dir(&mut self) {
        if let Some(i) = self.dir_list.state.selected() {
            let new_path = self.dir_list.items[i].path.clone();
            if new_path == "../" {
                self.move_up_dir();
            } else {
                let path_type = Path::new(&new_path);
                if path_type.is_dir() {
                    self.dir_list = DirList::new(path_type);
                    self.curr_dir = new_path;
                    self.dir_list.state.select_first();
                }
            }
        }
    }

    fn delete_selected(&mut self){
        if let Some(i) = self.dir_list.state.selected() {
            let remove_path = self.dir_list.items[i].path.clone();
            if remove_path != "../" {
                let remove_path = Path::new(&remove_path);
                if remove_path.is_file(){
                    match fs::remove_file(remove_path){
                        Ok(_) => {},
                        Err(e) => println!("Error deleting file {}", e),
                    }
                } else if remove_path.is_dir(){
                    // create warning screen
                }
                self.dir_list = DirList::new(Path::new(&self.curr_dir));
            }
        }
    }

    fn handle_command(&mut self, cmd: &str) {
        let string_cmd = String::from(cmd);
        if string_cmd.starts_with("Enter File Name:") {
            let file_name : Vec<&str>= string_cmd.split(":").collect();
            let curr_path = Path::new(&self.curr_dir).join(file_name.get(1).unwrap());
            self.create_file(&curr_path);
        } else if string_cmd.starts_with("Enter File Name:") {
            let dir_name : Vec<&str>= string_cmd.split(":").collect();
            let curr_path = Path::new(&self.curr_dir).join(dir_name.get(1).unwrap());
            self.create_dir(&curr_path);
        } else {
            match cmd {
                ":q" => self.should_exit = true,
                _ => {}
            }
        }
    }

    fn create_file(&mut self, file_path: &Path){
        match fs::File::create(file_path){
            Ok(_) => {},
            Err(e) => println!("Failed to create file {}", e),
        }
        self.dir_list = DirList::new(Path::new(&self.curr_dir));
    }

    fn create_dir(&mut self, dir_path : &Path){
        match fs::create_dir(dir_path){
            Ok(_) => {},
            Err(e) => println!("Failed to create directory {}", e),
        }
        self.dir_list = DirList::new(Path::new(&self.curr_dir));
    }

    pub fn commit(&mut self) {
        // FIND PATHS & COMPLETE ACTIONS -> MIGHT NEED TO BE HASH MAP INSTEAD (old_path:
        // new_path/state, e.g., /home/oliver/test.txt: deleted, home/oliver/filename.py:
        // home/oliver/newfilename.py, /home/oliver/test: created, etc.)
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [header, main, footer] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        let [list_area, _item_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Fill(1)]).areas(main);
        App::render_header(header, buf);
        self.render_list(list_area, buf);
        self.render_footer(footer, buf);
        //self.render_selected_item(item_area, buf);
    }
}

// rendering logic
impl App {
    fn render_header(area: Rect, buf: &mut Buffer) {
        Paragraph::new("ShellFM")
            .bold()
            .centered()
            .render(area, buf);
    }

    fn render_footer(&mut self, area: Rect, buf: &mut Buffer) {
        let [left, _right] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Fill(2)]).areas(area);
        self.shellpos = Some(area);
        Paragraph::new(format!("{}", self.mode))
            .set_style(Style::new().fg(TEXT_COLOR))
            .left_aligned()
            .render(left, buf);
        self.console.render(area, buf);
    }

    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw(self.curr_dir.clone()).centered())
            .borders(Borders::TOP)
            .border_set(symbols::border::EMPTY)
            .border_style(HEADER_STYLE)
            .bg(ROW_BG);

        let items: Vec<ListItem> = self
            .dir_list
            .items
            .iter()
            .enumerate()
            .map(|(i, dir_item)| {
                let color = alt_row_color(i);
                ListItem::from(dir_item).bg(color)
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol(">>")
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, &mut self.dir_list.state);
    }
}

const fn alt_row_color(i: usize) -> Color {
    if i % 2 == 0 { ROW_BG } else { ALT_ROW_BG }
}

// dir to list item
impl From<&Dir> for ListItem<'_> {
    fn from(value: &Dir) -> Self {
        let line = Line::styled(format!("{}", value.display), TEXT_COLOR);
        ListItem::new(line)
    }
}
