use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct AppLayout {
    pub explorer: Rect,
    pub editor_area: Rect,
    pub tab_bar: Rect,
    pub terminal_area: Rect,
    pub outline_area: Rect,
    pub system_area: Rect,
    pub status_bar: Rect,
    
    // Keeping these to avoid compilation errors elsewhere if they are still accessed
    pub test_runner_area: Rect,
    pub diagnostics_area: Rect,
}

pub fn calculate_layout(
    area: Rect,
    show_explorer: bool,
    show_terminal: bool,
    show_test_runner: bool,
    show_diagnostics: bool,
) -> AppLayout {
    // Top-level split: Main Content vs Status Bar
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    let main_area = main_chunks[0];
    let status_bar = main_chunks[1];

    // 3-column split: Left (20%), Middle (60%), Right (20%)
    let col_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(main_area);

    let explorer = if show_explorer { col_chunks[0] } else { Rect::default() };
    
    // Middle column
    let mid_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(col_chunks[1]);

    let mid_top_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(mid_chunks[0]);
        
    let tab_bar = mid_top_chunks[0];
    let editor_area = mid_top_chunks[1];
    let terminal_area = mid_chunks[1];

    // Right column
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(col_chunks[2]);

    let outline_area = right_chunks[0];
    let system_area = right_chunks[1];

    AppLayout {
        explorer,
        editor_area,
        tab_bar,
        status_bar,
        terminal_area,
        outline_area,
        system_area,
        test_runner_area: Rect::default(),
        diagnostics_area: Rect::default(),
    }
}
