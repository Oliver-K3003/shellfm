use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum Action {
    // DirList
    DirMode,
    SelectNext,
    SelectPrev,
    SelectFirst,
    SelectLast,
    MoveUpDir,
    MoveDownDir,
    // Console
    CmdMode,
    ShowConsole,
    // General
    Tick,
    Render,
    Resize(u16, u16),
    Suspend,
    Resume,
    Quit,
    ClearScreen,
    Error(String),
    Help,
}
