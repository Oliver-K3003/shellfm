use std::fs;
use std::path::Path;
use ratatui::widgets::ListState;

#[derive(Debug)]
pub struct Dir{
    pub path: String,
    pub display: String,
}

impl Dir {
    pub fn new(path: String, display: String) -> Self{
        Self {
            path,
            display,
        }
    }
}

pub struct DirList {
    pub items: Vec<Dir>,
    pub state: ListState,
}

impl FromIterator<(String, String)> for DirList{
    fn from_iter<I: IntoIterator<Item = (String, String)>>(iter: I) -> Self{
        let items = iter.into_iter().map(|(path, display)| Dir::new(path, display)).collect();
        let state = ListState::default();
        Self {
            items,
            state
        }
    }
}

impl DirList {
    pub fn new(path: &Path) -> Self {
        Self::from_iter(Self::get_dir(path))
    }

    fn get_dir(path: &Path) -> Vec<(String, String)> {
        let mut paths : Vec<(String, String)> = Vec::new();
        if let Some(_) = path.parent() {
            paths.push((String::from("../"), String::from("../")));
        }
        match fs::read_dir(path) {
            Ok(iter) => {
                for path in iter {
                    let p = path.unwrap().path();
                    paths.push(
                        (p.display().to_string(),
                         p.file_name().unwrap().to_str().unwrap().to_string())
                    );
                }
            }
            Err(err) => {
                println!("failed to open file: {}", err);
            }
        }
        paths
    }
}
