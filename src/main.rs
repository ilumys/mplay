mod library;
mod ui;

fn main() {
    library::build_library();
    let terminal = ratatui::init();
    ui::run(terminal);
    ratatui::restore();
}
