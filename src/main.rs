use std::{error::Error, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Tabs, Cell, Row, Table, TableState},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    
    Frame, Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

struct TableData<'a> {
    state: TableState,
    items: Vec<Vec<&'a str>>,
}

struct TimeData<'a> {
    pub titles: Vec<&'a str>,
    pub index: usize,
}

impl <'a> TimeData<'a> {
    fn new(values: Vec<&'a str>) -> Self {
        TimeData {
            titles: values,
            index: 0,
        }
    }

    fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }

    fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        }
        else {
            self.index = self.titles.len() - 1;
        }
    }
} 

impl<'a> TableData<'a> {
    fn new() -> Self {
        TableData {
            state: TableState::default(),
            items: vec![
                vec!["Row11", "Row12", "Row13"],
                vec!["Row21", "Row22", "Row23"],
                vec!["Row31", "Row32", "Row33"],
                vec!["Row41", "Row42", "Row43"],
                vec!["Row51", "Row52", "Row53"],
            ]
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

enum SelectedTab {
    Years,
    Months,
    Table,
}

impl SelectedTab {
    fn change_tab_up(self) -> Self {
        let to_return;
        match &self {
            SelectedTab::Years => to_return = SelectedTab::Table,
            SelectedTab::Months => to_return = SelectedTab::Years,
            SelectedTab::Table => to_return = SelectedTab::Months
        };
        to_return
    }

    fn change_tab_down(self) -> Self {
        let to_return;
        match &self {
            SelectedTab::Years => to_return = SelectedTab::Months,
            SelectedTab::Months => to_return = SelectedTab::Table,
            SelectedTab::Table => to_return = SelectedTab::Years
        };
        to_return
    }
}

fn main() -> Result<(), Box<dyn Error>>{
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let months = TimeData::new(vec!["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"]);
    let years = TimeData::new(vec!["2021", "2022", "2023", "2024", "2025", "2026"]);
    let tui_table = TableData::new();
    let res = run_app(&mut terminal, months, years, tui_table);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }
    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut months: TimeData, mut years: TimeData, mut table: TableData) -> io::Result<()> {
    let mut selected_tab = SelectedTab::Months;

    loop {
        terminal.draw(|f| ui(f, &months, &years, &mut table, &selected_tab))?;
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Right => {
                    match &selected_tab {
                        SelectedTab::Months => months.next(),
                        SelectedTab::Years => {
                            years.next();
                            months.index = 0;
                        },
                        _ => {}
                    }
                },
                KeyCode::Left => {
                    match &selected_tab {
                        SelectedTab::Months => months.previous(),
                        SelectedTab::Years => {
                            years.previous();
                            months.index = 0;
                        },
                        _ => {}
                    }
                },
                KeyCode::Up => {
                    match &selected_tab{
                        SelectedTab::Table => {
                            if table.state.selected() == Some(0) {
                                selected_tab = SelectedTab::Months;
                                table.state.select(Some(0));
                            }
                            else {
                                table.previous();
                            }
                        },
                        SelectedTab::Years => {
                            table.state.select(Some(table.items.len() - 1));
                            selected_tab = selected_tab.change_tab_up();
                        }
                        _ => selected_tab = selected_tab.change_tab_up()
                    }
                },
                KeyCode::Down => {
                    match &selected_tab {
                        SelectedTab::Table => {
                            if table.state.selected() == Some(table.items.len() - 1) {
                                selected_tab = SelectedTab::Years;
                                table.state.select(Some(0));
                            }
                            else {
                                table.next();
                            }
                        }
                        _ => selected_tab = selected_tab.change_tab_down(),
                    }
                    
                },
                _ => {}
            };
        };
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, months: &TimeData, years: &TimeData, table: &mut TableData, cu_tab: &SelectedTab) {
    let size = f.size();
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(Color::LightBlue);
    let header_cells = ["Header 1", "Header 2", "Header 3"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));

    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);

        let rows = table.items.iter().map(|item| {
            let height = item
                .iter()
                .map(|content| content.chars().filter(|c| *c == '\n').count())
                .max()
                .unwrap_or(0)
                + 1;
            let cells = item.iter().map(|c| Cell::from(*c));
            Row::new(cells).height(height as u16).bottom_margin(1)
        });

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(5)
        .constraints([Constraint::Length(3), Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(size);

    let block = Block::default().style(Style::default().bg(Color::White).fg(Color::Green));
    f.render_widget(block, size);

    let month_titles = months.titles.iter().map(|t| {
        let (first, rest) = t.split_at(3);
        Spans::from(vec![
            Span::styled(first, Style::default().fg(Color::Blue)),
            Span::styled(rest, Style::default().fg(Color::Green))
        ])
    })
    .collect();

    let year_titles = years.titles.iter().map(|t| {
        let (first, rest) = t.split_at(1);
        Spans::from(vec![
            Span::styled(first, Style::default().fg(Color::Blue)),
            Span::styled(rest, Style::default().fg(Color::Green))
        ])
    })
    .collect();

    let mut month_tab = Tabs::new(month_titles)
        .block(Block::default().borders(Borders::ALL).title("Months"))
        .select(months.index)
        .style(Style::default().fg(Color::Green))
        .highlight_style(Style::default()
        .add_modifier(Modifier::BOLD)
        .bg(Color::Black));
        
    
    let mut year_tab = Tabs::new(year_titles)
        .block(Block::default().borders(Borders::ALL).title("Years"))
        .select(years.index)
        .style(Style::default().fg(Color::Green))
        .highlight_style(Style::default()
        .add_modifier(Modifier::BOLD)
        .bg(Color::Black));

    let mut table_area = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Table"))
        .widths(&[
            Constraint::Percentage(50),
            Constraint::Length(30),
            Constraint::Min(10),
        ]);
        
    match cu_tab {
        SelectedTab::Months => {
            month_tab = month_tab.highlight_style(Style::default()
            .add_modifier(Modifier::BOLD)
            .bg(Color::LightGreen));
        }

        SelectedTab::Years => {
            year_tab = year_tab.highlight_style(Style::default()
            .add_modifier(Modifier::BOLD)
            .bg(Color::LightGreen));
        }

        SelectedTab::Table => {
            table_area = table_area.highlight_style(selected_style)
            .highlight_symbol(">> ")
        }
    }

    f.render_widget(month_tab, chunks[1]);
    f.render_widget(year_tab, chunks[0]);
    f.render_stateful_widget(table_area, chunks[2], &mut table.state)
}