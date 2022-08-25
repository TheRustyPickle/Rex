use tui::{
    symbols,
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Span},
    widgets::{Block, Chart, Dataset, Axis, GraphType},
    Frame,
};

pub fn chart_ui<B: Backend>(f: &mut Frame<B>) {
    let size = f.size();

    // divide the terminal into various chunks to draw the interface. This is a vertical chunk
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Percentage(70),
            ]
            .as_ref(),
        )
        .split(size);
    let block = Block::default().style(
        Style::default()
            .bg(Color::Rgb(255, 255, 255))
            .fg(Color::Rgb(50, 205, 50)),
    );
    f.render_widget(block, size);

    let datasets = vec![
        Dataset::default()
            .name("data1")
            .marker(symbols::Marker::Dot)
            .graph_type(GraphType::Scatter)
            .style(Style::default().fg(Color::Cyan))
            .data(&[(0.0, 5.0), (1.0, 6.0), (1.5, 6.434)]),
        Dataset::default()
            .name("data2")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Magenta))
            .data(&[(4.0, 5.0), (5.0, 8.0), (7.66, 13.5)]),
    ];
    let chart = Chart::new(datasets)
        .block(Block::default().title("Chart"))
        .x_axis(Axis::default()
            .title(Span::styled("X Axis", Style::default().fg(Color::Red)))
            .style(Style::default().fg(Color::White))
            .bounds([0.0, 10.0])
            .labels(["0.0", "5.0", "10.0"].iter().cloned().map(Span::from).collect()))
        .y_axis(Axis::default()
            .title(Span::styled("Y Axis", Style::default().fg(Color::Red)))
            .style(Style::default().fg(Color::White))
            .bounds([0.0, 10.0])
            .labels(["0.0", "5.0", "10.0"].iter().cloned().map(Span::from).collect()));

    f.render_widget(chart, chunks[0]);
}