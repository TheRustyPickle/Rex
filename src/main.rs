use std::{error::Error, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Tabs},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    
    Frame, Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

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

enum SelectedTab {
    Months,
    Years,
}

impl SelectedTab {
    fn change_tab(self) -> Self {
        let to_return;
        match &self {
            SelectedTab::Months => to_return = SelectedTab::Years,
            SelectedTab::Years => to_return = SelectedTab::Months,
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
    let res = run_app(&mut terminal, months, years);

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

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut months: TimeData, mut years: TimeData ) -> io::Result<()> {
    let mut selected_tab = SelectedTab::Months;

    loop {
        terminal.draw(|f| ui(f, &months, &years, &selected_tab))?;
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
                    }
                },
                KeyCode::Left => {
                    match &selected_tab {
                        SelectedTab::Months => months.previous(),
                        SelectedTab::Years => {
                            years.previous();
                            months.index = 0;
                        },
                    }
                },
                KeyCode::Up => selected_tab = selected_tab.change_tab(),
                KeyCode::Down => selected_tab = selected_tab.change_tab(),
                _ => {}
            };
        };
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, months: &TimeData, years: &TimeData, cu_tab: &SelectedTab) {
    let size = f.size();
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
        .style(Style::default().fg(Color::Green));
        
    
    let mut year_tab = Tabs::new(year_titles)
        .block(Block::default().borders(Borders::ALL).title("Years"))
        .select(years.index)
        .style(Style::default().fg(Color::Green));
        
    match cu_tab {
        SelectedTab::Months => {
            month_tab = month_tab.highlight_style(Style::default()
            .add_modifier(Modifier::BOLD)
            .bg(Color::Black));
        }

        SelectedTab::Years => {
            year_tab = year_tab.highlight_style(Style::default()
            .add_modifier(Modifier::BOLD)
            .bg(Color::Black));
        }
    }

    f.render_widget(month_tab, chunks[1]);
    f.render_widget(year_tab, chunks[0]);

    let inner = Block::default().title("Data").borders(Borders::ALL);
    f.render_widget(inner, chunks[2]);
}