use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, BorderType, Gauge, List, ListItem, Sparkline, Widget},
};

pub fn render_outline(area: Rect, frame: &mut ratatui::Frame) {
    let items = vec![
        ListItem::new(" fn show_banner()").style(Style::default().fg(Color::Cyan)),
        ListItem::new(" fn main()").style(Style::default().fg(Color::Magenta)),
        ListItem::new(" struct App").style(Style::default().fg(Color::Cyan)),
        ListItem::new(" impl App").style(Style::default().fg(Color::Magenta)),
        ListItem::new(" fn render()").style(Style::default().fg(Color::Cyan)),
        ListItem::new(" fn handle_input()").style(Style::default().fg(Color::Magenta)),
    ];

    let list = List::new(items).block(
        Block::default()
            .title(" Outline ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(list, area);
}

pub fn render_system_monitor(area: Rect, frame: &mut ratatui::Frame) {
    let block = Block::default()
        .title(" System ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Magenta));

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Mem Gauge
            Constraint::Length(3), // CPU Sparkline
            Constraint::Length(3), // Net Sparkline
        ])
        .split(inner_area);

    let mem_gauge = Gauge::default()
        .block(Block::default().title("Mem: 8.2G / 24G"))
        .gauge_style(Style::default().fg(Color::Cyan).bg(Color::DarkGray))
        .percent(34)
        .label("34%");
    frame.render_widget(mem_gauge, chunks[0]);

    let cpu_data = [20, 35, 40, 30, 50, 70, 80, 50, 40, 45, 60, 55, 30];
    let cpu_sparkline = Sparkline::default()
        .block(Block::default().title("CPU"))
        .data(&cpu_data)
        .style(Style::default().fg(Color::Magenta))
        .bar_set(symbols::bar::NINE_LEVELS);
    frame.render_widget(cpu_sparkline, chunks[1]);

    let net_data = [10, 15, 20, 25, 40, 30, 20, 10, 5, 20, 30, 40, 50];
    let net_sparkline = Sparkline::default()
        .block(Block::default().title("Net"))
        .data(&net_data)
        .style(Style::default().fg(Color::Cyan))
        .bar_set(symbols::bar::NINE_LEVELS);
    frame.render_widget(net_sparkline, chunks[2]);
}
