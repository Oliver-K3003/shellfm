pub mod app;

use std::error;

use app::App;

/*
TODO
    [x] - Make list state dependent
    [x] - Get current dir (when opening)
    [x] - j/k or up/down for nav between files
    [ ] - h/l or right/left for nav up/down directories
    [ ] - preview window
    [ ] - default programs for opening
    [ ] - search bar (different functions)
        [ ] - entire file system
        [ ] - current dir (w or w/o subdirs)
        [ ] - grep within file
        [ ] - within upper dir/specified dir
    [ ] - add files, dirs
        [ ] - make file & open default editor
    [ ] - remove files, dirs
    [ ] - rename files, dirs
    [ ] - move files, dirs
    [ ] - copy files, dirs
 */

fn main() -> Result<(), Box<dyn error::Error>> {
    let terminal = ratatui::init();
    let app_result = App::new().run(terminal);
    ratatui::restore();
    app_result
}
