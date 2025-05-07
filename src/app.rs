use ratatui::{
    DefaultTerminal,
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize, palette::tailwind},
    symbols,
    text::Line,
    widgets::{
        Block, Borders, HighlightSpacing, List, ListItem, Paragraph, StatefulWidget, Widget,
    },
};
use std::{env, error, path::Path, fmt::{self, Display}};

pub mod display;
use display::{Dir, DirList};

// set global styles
const HEADER_STYLE: Style = Style::new()
    .fg(tailwind::ZINC.c300)
    .bg(tailwind::SLATE.c700);
const ROW_BG: Color = tailwind::SLATE.c800;
const ALT_ROW_BG: Color = tailwind::SLATE.c900;
const SELECTED_STYLE: Style = Style::new().bg(tailwind::SLATE.c600);
const TEXT_COLOR: Color = tailwind::ZINC.c300;

pub enum Mode {
    Normal,
    Visual,
    Insert,
    Command,
}

impl Display for Mode{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        let val: &str = match self{
            Mode::Normal => "Normal",
            Mode::Insert => "Insert",
            Mode::Visual => "Visual",
            _ => ""
        };
        write!(f, "{}", val)
    }
}

pub enum State {
    Editing,
}

pub struct App {
    pub mode: Mode,
    pub state: Option<State>,
    pub dir_list: DirList,
    pub should_exit: bool,
    pub curr_dir: String,
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
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<(), Box<dyn error::Error>> {
        self.dir_list.state.select_first();
        while !self.should_exit {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
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
                    KeyCode::Char('q') => self.should_exit = true,
                    KeyCode::Char('g') | KeyCode::Home => self.select_first(),
                    KeyCode::Char('G') | KeyCode::End => self.select_last(),
                    KeyCode::Char('i') => self.enter_insert('i'),
                    KeyCode::Char('a') => self.enter_insert('a'),
                    _ => {}
                };
            }
            Mode::Insert => {
                match key.code {
                    KeyCode::Char('o') => {},
                    KeyCode::Esc => self.mode = Mode::Normal,
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn enter_insert(&mut self, inp: char){
        self.mode = Mode::Insert;
        if inp == 'i'{
            self.dir_list.state.select_first();
        } else{
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
            let path_type = Path::new(&new_path);
            if path_type.is_dir() {
                self.dir_list = DirList::new(path_type);
                self.curr_dir = new_path;
                self.dir_list.state.select_first();
            }
        }
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

    fn render_footer(&mut self, area: Rect, buf: &mut Buffer){
        Paragraph::new(format!("{}", self.mode))
            .centered()
            .render(area, buf);
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

impl From<&Dir> for ListItem<'_> {
    fn from(value: &Dir) -> Self {
        let line = Line::styled(format!("{}", value.display), TEXT_COLOR);
        ListItem::new(line)
    }
}
