use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct AppLayout {
    pub explorer: Rect,
    pub editor_area: Rect,
    pub tab_bar: Rect,
    pub status_bar: Rect,
    pub terminal_area: Rect,
    pub test_runner_area: Rect,
}

pub fn calculate_layout(area: Rect, show_explorer: bool, show_terminal: bool, show_test_runner: bool) -> AppLayout {
    // Top-level split: Main Content vs Status Bar
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    let main_area = main_chunks[0];
    let status_bar = main_chunks[1];

    // Split Main Content: Explorer vs Right Pane
    let (explorer, right_pane) = if show_explorer {
        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(30), Constraint::Min(0)])
            .split(main_area);
        (horizontal[0], horizontal[1])
    } else {
        (Rect::default(), main_area)
    };

    // Split Right Pane: Tab Bar vs Rest
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(right_pane);

    let tab_bar = right_chunks[0];
    let mut rest_area = right_chunks[1];

    let mut test_runner_area = Rect::default();
    if show_test_runner {
        let splits = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(rest_area);
        rest_area = splits[0];
        test_runner_area = splits[1];
    }

    let (editor_area, terminal_area) = if show_terminal {
        let terminal_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(rest_area);
        (terminal_chunks[0], terminal_chunks[1])
    } else {
        (rest_area, Rect::default())
    };

    AppLayout {
        explorer,
        editor_area,
        tab_bar,
        status_bar,
        terminal_area,
        test_runner_area,
    }
}
