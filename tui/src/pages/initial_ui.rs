use chrono::{Days, Local, NaiveDate};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Cell, Paragraph, Row, Table, Wrap};
use rex_app::conn::{DbConn, FullRecurringTx};
use thousands::Separable;

use crate::pages::{A, F, J, Q, R, U, W, Y, Z};
use crate::theme::Theme;
use crate::utility::{create_bolded_text, main_block, styled_block};

/// How many days ahead the startup screen looks for upcoming recurring transactions
const UPCOMING_DAYS: u64 = 7;
/// The most upcoming recurring transactions listed on the startup screen
const UPCOMING_MAX_ROWS: usize = 10;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Sign {
    Positive,
    Negative,
    Neutral,
}

pub struct UpcomingRow {
    pub date: String,
    pub details: String,
    pub amount: String,
    pub sign: Sign,
}

/// Everything the startup screen shows besides the static key reference. Loaded once
/// before the UI loop starts since the page is redrawn every few milliseconds.
pub struct InitialData {
    pub recurring_added: usize,
    /// Method name, formatted balance, whether the balance is negative
    pub balances: Vec<(String, String, bool)>,
    pub total: Option<(String, bool)>,
    pub upcoming: Vec<UpcomingRow>,
    pub paused_count: usize,
}

impl InitialData {
    pub fn load(conn: &mut DbConn, recurring: &[FullRecurringTx], recurring_added: usize) -> Self {
        let final_balances = conn.get_final_balances().unwrap_or_default();

        let mut balances = Vec::new();
        let mut total_cents = 0_i64;

        for method in conn.get_tx_methods_sorted() {
            let cents = final_balances.get(&method.id).map_or(0, |b| b.balance);
            total_cents += cents;
            balances.push((method.name.clone(), format_cents(cents), cents < 0));
        }

        let total = (balances.len() > 1).then(|| (format_cents(total_cents), total_cents < 0));

        let today = Local::now().date_naive();

        let mut due_soon: Vec<&FullRecurringTx> = recurring
            .iter()
            .filter(|r| {
                is_upcoming(
                    r.next_recurring_date,
                    r.end_date,
                    r.is_paused,
                    today,
                    UPCOMING_DAYS,
                )
            })
            .collect();

        due_soon.sort_by_key(|r| r.next_recurring_date);

        let upcoming = due_soon
            .into_iter()
            .take(UPCOMING_MAX_ROWS)
            .map(|r| {
                let tx_type = r.tx_type.to_string();

                let sign = match tx_type.as_str() {
                    "Income" | "Borrow" | "Lend Repay" => Sign::Positive,
                    "Expense" | "Lend" | "Borrow Repay" => Sign::Negative,
                    _ => Sign::Neutral,
                };

                UpcomingRow {
                    date: r.next_recurring_date.format("%a %b %d").to_string(),
                    details: r.details.clone().unwrap_or(tx_type),
                    amount: format!("{:.2}", r.amount.dollar()).separate_with_commas(),
                    sign,
                }
            })
            .collect();

        Self {
            recurring_added,
            balances,
            total,
            upcoming,
            paused_count: recurring.iter().filter(|r| r.is_paused).count(),
        }
    }
}

fn format_cents(cents: i64) -> String {
    format!("{:.2}", cents as f64 / 100.0).separate_with_commas()
}

/// Whether a recurring rule's next occurrence falls within the next `days` days and will
/// actually fire (not paused, not past its end date)
fn is_upcoming(
    next: NaiveDate,
    end_date: Option<NaiveDate>,
    is_paused: bool,
    today: NaiveDate,
    days: u64,
) -> bool {
    if is_paused || end_date.is_some_and(|end| next > end) {
        return false;
    }

    next >= today && next <= today + Days::new(days)
}

/// Tables inherit their block's border-coloured foreground for unstyled cells, so every row
/// gets the theme's text colour explicitly, like the other pages do.
fn row_style(theme: &Theme) -> Style {
    Style::default().bg(theme.background()).fg(theme.text())
}

fn sign_style(sign: Sign, theme: &Theme) -> Style {
    match sign {
        Sign::Positive => Style::default().fg(theme.positive()),
        Sign::Negative => Style::default().fg(theme.negative()),
        Sign::Neutral => Style::default().fg(theme.text()),
    }
}

/// The function draws the Initial page of the interface.
pub fn initial_ui(f: &mut Frame, start_from: usize, data: &InitialData, theme: &Theme) {
    let size = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(8),
            Constraint::Length(1),
            Constraint::Min(5),
        ])
        .split(size);

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(chunks[2]);

    let left_column = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(columns[0]);

    let right_column = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(11), Constraint::Min(0)])
        .split(columns[1]);

    f.render_widget(main_block(theme), size);

    // This is the text that is shown in the startup which is the project's name in ASCII format.
    let text = r"   _____    ______  __   __
  |  __ \  |  ____| \ \ / /
  | |__) | | |__     \ V / 
  |  _  /  |  __|     > <  
  | | \ \  | |____   / . \ 
  |_|  \_\ |______| /_/ \_\"
        .to_string();

    // To animate it, the text is split by \n and each line reveals `total_to_add`
    // chars starting at `start_from`, wrapping around to the beginning of the line
    // when the end is reached
    let split_text = text.split('\n').collect::<Vec<&str>>();
    let mut upper_text = String::new();

    for line in split_text {
        // A line of 20 chars at index 15 takes chars 15-20 and then 0-3
        // This var stores how many to take from 0 index
        let mut to_add_from_start = 0;
        // amount of chars per line
        let mut total_to_add = 10;

        if start_from + total_to_add > line.len() {
            let extra_index = (start_from + total_to_add) - line.len();
            // Take that many chars from the start instead, since taking them from
            // the starting point would go out of bounds
            total_to_add -= extra_index;
            to_add_from_start += extra_index;
        }

        // Go through each char of the line
        for (index, char) in line.chars().enumerate() {
            if to_add_from_start != 0 {
                // Wrapping around, so keep the chars until the count runs out
                upper_text.push(char);
                to_add_from_start -= 1;
            } else if total_to_add != 0 && index >= start_from {
                // Past the start point and the limit isn't reached yet
                upper_text.push(char);
                total_to_add -= 1;
            } else if index != start_from || total_to_add == 0 {
                // Everything past the 10 char limit stays blank
                upper_text.push(' ');
            }
        }
        upper_text.push('\n');
    }

    let unmodified_first_help = format!(
        "{F}
{A}
{R}
{Z}
{Y}
{W}
{U}
{J}
{Q}"
    );

    let unmodified_second_help = "H: Show the keys of the page you are currently on
Arrow Up/Down: Cycle between widgets
Arrow Left/Right: Cycle values of a widget
Number keys: Jump to a field on Add Transaction/Search/Recurring pages
T: Cycle through themes
X: Change sort type on Summary/Change date type on Search page
Double r/R/Z: Hide chart top widgets/legends or summary top widget";

    // create_bolded_text does the bolding while rendering
    let first_text = create_bolded_text(&unmodified_first_help);
    let second_text = create_bolded_text(unmodified_second_help);

    let middle_text = "Press Any Key To Continue";

    let paragraph = Paragraph::new(upper_text)
        .style(
            Style::default()
                .bg(theme.background())
                .fg(theme.text())
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);

    let paragraph_2 = Paragraph::new(middle_text)
        .style(
            Style::default()
                .bg(theme.background())
                .fg(theme.negative())
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);

    let help_1 = Paragraph::new(first_text)
        .style(Style::default().bg(theme.background()).fg(theme.text()))
        .block(styled_block("Page Keys", theme))
        .wrap(Wrap { trim: true });

    let help_2 = Paragraph::new(second_text)
        .style(Style::default().bg(theme.background()).fg(theme.text()))
        .block(styled_block("Other Keys", theme))
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, chunks[0]);
    f.render_widget(paragraph_2, chunks[1]);
    render_balances(f, left_column[0], data, theme);
    render_recurring(f, left_column[1], data, theme);
    f.render_widget(help_1, right_column[0]);
    f.render_widget(help_2, right_column[1]);
}

fn render_balances(f: &mut Frame, area: ratatui::layout::Rect, data: &InitialData, theme: &Theme) {
    let block = styled_block("Balances", theme);

    if data.balances.is_empty() {
        let empty = Paragraph::new("No transaction methods yet. Press J to add one.")
            .style(Style::default().bg(theme.background()).fg(theme.text()))
            .block(block)
            .wrap(Wrap { trim: true });
        f.render_widget(empty, area);
        return;
    }

    let balance_cell = |value: &str, negative: bool, bold: bool| {
        let color = if negative {
            theme.negative()
        } else {
            theme.text()
        };
        let mut style = Style::default().fg(color);

        if bold {
            style = style.add_modifier(Modifier::BOLD);
        }

        Cell::from(Line::from(Span::styled(value.to_string(), style)).right_aligned())
    };

    let mut rows: Vec<Row> = data
        .balances
        .iter()
        .map(|(name, balance, negative)| {
            Row::new([
                Cell::from(name.clone()),
                balance_cell(balance, *negative, false),
            ])
            .style(row_style(theme))
        })
        .collect();

    if let Some((total, negative)) = &data.total {
        rows.push(
            Row::new([
                Cell::from(Span::styled(
                    "Total",
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                balance_cell(total, *negative, true),
            ])
            .style(row_style(theme)),
        );
    }

    let table = Table::new(
        rows,
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    )
    .style(Style::default().bg(theme.background()).fg(theme.text()))
    .block(block);

    f.render_widget(table, area);
}

fn render_recurring(f: &mut Frame, area: ratatui::layout::Rect, data: &InitialData, theme: &Theme) {
    let title = format!("Recurring: next {UPCOMING_DAYS} days");
    let block = styled_block(&title, theme);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(0)])
        .split(inner);

    let added_text = match data.recurring_added {
        0 => "No recurring transactions were due on startup".to_string(),
        1 => "1 recurring transaction was added on startup".to_string(),
        n => format!("{n} recurring transactions were added on startup"),
    };

    let added_style = if data.recurring_added > 0 {
        Style::default()
            .fg(theme.positive())
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text())
    };

    let mut status_lines = vec![Line::from(Span::styled(added_text, added_style))];

    if data.paused_count > 0 {
        status_lines.push(Line::from(format!("{} paused", data.paused_count)));
    }

    f.render_widget(
        Paragraph::new(status_lines)
            .style(Style::default().bg(theme.background()).fg(theme.text())),
        sections[0],
    );

    if data.upcoming.is_empty() {
        f.render_widget(
            Paragraph::new("Nothing coming up. Press U to manage recurring transactions.")
                .style(Style::default().bg(theme.background()).fg(theme.text()))
                .wrap(Wrap { trim: true }),
            sections[1],
        );
        return;
    }

    let rows = data.upcoming.iter().map(|row| {
        Row::new([
            Cell::from(row.date.clone()),
            Cell::from(row.details.clone()),
            Cell::from(
                Line::from(Span::styled(
                    row.amount.clone(),
                    sign_style(row.sign, theme),
                ))
                .right_aligned(),
            ),
        ])
        .style(row_style(theme))
    });

    let table = Table::new(
        rows,
        [
            Constraint::Length(11),
            Constraint::Min(10),
            Constraint::Length(12),
        ],
    )
    .style(Style::default().bg(theme.background()).fg(theme.text()));

    f.render_widget(table, sections[1]);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn upcoming_includes_today_and_the_last_day_of_the_window() {
        let today = date(2026, 9, 29);
        assert!(is_upcoming(today, None, false, today, 7));
        assert!(is_upcoming(date(2026, 10, 6), None, false, today, 7));
    }

    #[test]
    fn upcoming_excludes_outside_the_window() {
        let today = date(2026, 9, 29);
        assert!(!is_upcoming(date(2026, 10, 7), None, false, today, 7));
        assert!(!is_upcoming(date(2026, 9, 28), None, false, today, 7));
    }

    #[test]
    fn upcoming_excludes_paused_rules() {
        let today = date(2026, 9, 29);
        assert!(!is_upcoming(date(2026, 9, 30), None, true, today, 7));
    }

    #[test]
    fn upcoming_excludes_rules_past_their_end_date() {
        let today = date(2026, 9, 29);
        let next = date(2026, 10, 2);
        assert!(!is_upcoming(next, Some(date(2026, 10, 1)), false, today, 7));
        assert!(is_upcoming(next, Some(date(2026, 10, 2)), false, today, 7));
    }

    #[test]
    fn format_cents_adds_separators_and_keeps_sign() {
        assert_eq!(format_cents(123_456_789), "1,234,567.89");
        assert_eq!(format_cents(-250), "-2.50");
        assert_eq!(format_cents(0), "0.00");
    }

    fn sample_data() -> InitialData {
        InitialData {
            recurring_added: 3,
            balances: vec![
                ("Cash".to_string(), "1,200.50".to_string(), false),
                ("Bank".to_string(), "-40.00".to_string(), true),
            ],
            total: Some(("1,160.50".to_string(), false)),
            upcoming: vec![UpcomingRow {
                date: "Wed Sep 30".to_string(),
                details: "Rent".to_string(),
                amount: "800.00".to_string(),
                sign: Sign::Negative,
            }],
            paused_count: 1,
        }
    }

    fn render(width: u16, height: u16, data: &InitialData) -> String {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        let theme = Theme::new_index(0);
        terminal.draw(|f| initial_ui(f, 5, data, &theme)).unwrap();

        let buffer = terminal.backend().buffer().clone();
        (0..height)
            .map(|y| {
                (0..width)
                    .map(|x| buffer[(x, y)].symbol().to_string())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn renders_balances_recurring_and_keys() {
        let screen = render(120, 40, &sample_data());

        assert!(screen.contains("Balances"));
        assert!(screen.contains("1,200.50"));
        assert!(screen.contains("Total"));
        assert!(screen.contains("3 recurring transactions were added on startup"));
        assert!(screen.contains("1 paused"));
        assert!(screen.contains("Rent"));
        assert!(screen.contains("U: Recurring Transactions Page"));
    }

    #[test]
    fn renders_empty_states() {
        let data = InitialData {
            recurring_added: 0,
            balances: Vec::new(),
            total: None,
            upcoming: Vec::new(),
            paused_count: 0,
        };
        let screen = render(120, 40, &data);

        assert!(screen.contains("No transaction methods yet"));
        assert!(screen.contains("No recurring transactions were due on startup"));
        assert!(screen.contains("Nothing coming up"));
    }

    #[test]
    fn does_not_panic_on_tiny_terminals() {
        for (w, h) in [(1, 1), (20, 5), (40, 12), (80, 24)] {
            render(w, h, &sample_data());
        }
    }

    #[test]
    fn table_text_uses_the_text_colour_not_the_border_colour() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        for idx in 0..12 {
            let theme = Theme::new_index(idx);
            let mut terminal = Terminal::new(TestBackend::new(120, 36)).unwrap();
            let data = sample_data();
            terminal.draw(|f| initial_ui(f, 5, &data, &theme)).unwrap();
            let buffer = terminal.backend().buffer().clone();

            for word in ["Cash", "Total", "Rent", "Wed Sep 30"] {
                let (x, y) = (0..36u16)
                    .find_map(|y| {
                        let row: String = (0..120u16)
                            .map(|x| buffer[(x, y)].symbol().to_string())
                            .collect();
                        row.find(word).map(|x| (x as u16, y))
                    })
                    .unwrap();

                assert_eq!(buffer[(x, y)].fg, theme.text(), "{word} in theme {idx}");
            }
        }
    }
}
