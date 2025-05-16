use std::fs;
use std::path::Path;
use tokio::sync::mpsc::UnboundedSender;

use color_eyre::Result;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style, Stylize, palette::tailwind},
    symbols,
    text::Line,
    widgets::{Block, Borders, HighlightSpacing, List, ListItem, ListState},
};

use super::Component;
use crate::{action::Action, config::Config};

const SELECTED_STYLE: Style = Style::new().bg(tailwind::SLATE.c600);
const ROW_BG: Color = tailwind::SLATE.c800;
const ALT_ROW_BG: Color = tailwind::SLATE.c900;
const TEXT_COLOR: Color = tailwind::ZINC.c300;

#[derive(Debug, Clone)]
struct Dir {
    path: String,
    display: String,
}

impl Dir {
    fn new(path: String, display: String) -> Self {
        Self { path, display }
    }
}

#[derive(Debug, Clone)]
pub struct DirList {
    curr_dir: String,
    items: Vec<Dir>,
    state: ListState,
    config: Config,
    command_tx: Option<UnboundedSender<Action>>,
}

impl DirList {
    pub fn new(path: &Path) -> Self {
        let mut new_val = Self::from_iter(Self::get_dir(path).unwrap());
        new_val.curr_dir = path.to_str().unwrap().to_string();
        new_val
    }

    fn get_dir(path: &Path) -> Result<Vec<(String, String)>> {
        let mut paths: Vec<(String, String)> = Vec::new();
        if let Some(_) = path.parent() {
            paths.push((String::from("../"), String::from("../")));
        }
        let iter = fs::read_dir(path)?;
        for path in iter {
            let p = path.unwrap().path();
            paths.push((
                p.display().to_string(),
                p.file_name().unwrap().to_str().unwrap().to_string(),
            ));
        }
        Ok(paths)
    }

    fn get_dir_list(path: &Path) -> Result<Vec<Dir>> {
        let mut dir_list: Vec<Dir> = Vec::new();
        if let Some(_) = path.parent() {
            dir_list.push(Dir::new(String::from("../"),String::from("../")));
        }
        let iter = fs::read_dir(path)?;
        for path in iter {
            let p = path.unwrap().path();
            dir_list.push(Dir::new(
                p.display().to_string(),
                p.file_name().unwrap().to_str().unwrap().to_string(),
            ));
        }
        Ok(dir_list)
    }

    fn select_next(&mut self) {
        self.state.select_next();
    }

    fn select_prev(&mut self) {
        self.state.select_previous();
    }

    fn select_first(&mut self) {
        self.state.select_first();
    }

    fn select_last(&mut self) {
        self.state.select_last();
    }

    fn move_up_dir(&mut self) -> Result<()> {
        if let Some(parent) = Path::new(&self.curr_dir).parent() {
            self.items = Self::get_dir_list(&parent)?;
            self.curr_dir = parent.to_str().unwrap().to_string();
        }
        Ok(())
    }

    fn move_down_dir(&mut self) -> Result<()> {
        if let Some(i) = self.state.selected() {
            let new_path = self.items[i].path.clone();
            if new_path == "../" {
                self.move_up_dir()?;
            } else {
                let path_type = Path::new(&new_path);
                if path_type.is_dir() {
                    self.items = Self::get_dir_list(&path_type)?;
                    self.curr_dir = path_type.to_str().unwrap().to_string();
                }
            }
        }
        Ok(())
    }
}

impl FromIterator<(String, String)> for DirList {
    fn from_iter<T: IntoIterator<Item = (String, String)>>(iter: T) -> Self {
        let items: Vec<Dir> = iter
            .into_iter()
            .map(|(path, display)| Dir::new(path, display))
            .collect();
        let state = ListState::default();
        let config = Config::default();
        let command_tx = None;
        let curr_dir = String::new();
        Self {
            curr_dir,
            items,
            state,
            config,
            command_tx,
        }
    }
}

impl Component for DirList {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Tick => {}
            Action::Render => {}
            Action::SelectNext => self.select_next(),
            Action::SelectPrev => self.select_prev(),
            Action::SelectFirst => self.select_first(),
            Action::SelectLast => self.select_last(),
            Action::MoveUpDir => self.move_up_dir()?,
            Action::MoveDownDir => self.move_down_dir()?,
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let block = Block::new()
            .title(Line::raw(self.curr_dir.clone()).centered())
            .borders(Borders::TOP)
            .border_set(symbols::border::EMPTY)
            .bg(ROW_BG);

        let items: Vec<ListItem> = self
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
            .highlight_symbol(">>")
            .highlight_style(SELECTED_STYLE)
            .highlight_spacing(HighlightSpacing::Always);

        frame.render_stateful_widget(list, area, &mut self.state);

        Ok(())
    }
}

impl From<&Dir> for ListItem<'_> {
    fn from(value: &Dir) -> Self {
        let line = Line::styled(format!("{}", value.display), TEXT_COLOR);
        ListItem::new(line)
    }
}

const fn alt_row_color(i: usize) -> Color {
    if i % 2 == 0 { ROW_BG } else { ALT_ROW_BG }
}
