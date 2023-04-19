use crate::chart_page::ChartData;
use crate::page_handler::{ChartTab, IndexedData};
use crate::utility::get_all_tx_methods;
use chrono::{naive::NaiveDate, Duration};
use rusqlite::Connection;
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Tabs};
use tui::{symbols, Frame};

/// Creates the balance chart from all the transactions in a given year
pub fn chart_ui<B: Backend>(
    f: &mut Frame<B>,
    months: &IndexedData,
    years: &IndexedData,
    mode_selection: &IndexedData,
    chart_data: &ChartData,
    current_page: &ChartTab,
    chart_hidden_mode: bool,
    loop_remaining: &mut Option<usize>,
    conn: &Connection,
) {
    let size = f.size();

    // divide the terminal into various chunks to draw the interface. This is a vertical chunk

    let mut main_layout = Layout::default().direction(Direction::Vertical).margin(2);

    if chart_hidden_mode {
        main_layout = main_layout.constraints([Constraint::Min(0)].as_ref())
    } else {
        match mode_selection.index {
            0 => {
                main_layout = main_layout.constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Min(0),
                    ]
                    .as_ref(),
                )
            }
            1 => {
                main_layout = main_layout.constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Min(0),
                    ]
                    .as_ref(),
                )
            }
            2 => {
                main_layout =
                    main_layout.constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            }
            _ => {}
        };
    }

    let chunks = main_layout.split(size);

    let block = Block::default().style(
        Style::default()
            .bg(Color::Rgb(255, 255, 255))
            .fg(Color::Rgb(50, 205, 50)),
    );
    f.render_widget(block, size);

    let month_titles = months
        .titles
        .iter()
        .map(|t| Spans::from(vec![Span::styled(t, Style::default().fg(Color::Blue))]))
        .collect();

    //color the first two letters of the year to blue
    let year_titles = years
        .titles
        .iter()
        .map(|t| Spans::from(vec![Span::styled(t, Style::default().fg(Color::Blue))]))
        .collect();

    let mode_selection_titles = mode_selection
        .titles
        .iter()
        .map(|t| Spans::from(vec![Span::styled(t, Style::default().fg(Color::Blue))]))
        .collect();

    // The default style for the select index in the month section if
    // the Month widget is not selected
    let mut month_tab = Tabs::new(month_titles)
        .block(Block::default().borders(Borders::ALL).title("Months"))
        .select(months.index)
        .style(Style::default().fg(Color::Rgb(50, 205, 50)))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Black),
        );

    // The default style for the select index in the year section if
    // the Year widget is not selected
    let mut year_tab = Tabs::new(year_titles)
        .block(Block::default().borders(Borders::ALL).title("Years"))
        .select(years.index)
        .style(Style::default().fg(Color::Rgb(50, 205, 50)))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Black),
        );

    let mut mode_selection_tab = Tabs::new(mode_selection_titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Mode Selection"),
        )
        .select(mode_selection.index)
        .style(Style::default().fg(Color::Rgb(50, 205, 50)))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Black),
        );

    // connect to the database and gather all the tx methods
    let all_tx_methods = get_all_tx_methods(conn);

    let mut datasets: Vec<Vec<(f64, f64)>> = Vec::new();
    let mut last_balances = Vec::new();

    // adding default initial value
    if chart_data.all_txs.is_empty() {
        for _i in &all_tx_methods {
            datasets.push(vec![(0.0, 0.0)]);
            last_balances.push(0.0);
        }
    }

    let mut lowest_balance = 0.0;
    let mut highest_balance = 0.0;

    let mut date_labels: Vec<String> = vec![];

    let mut current_axis = 0.0;

    // if there are no transactions, we will create an empty chart
    if !chart_data.all_txs.is_empty() {
        // contains all dates of the transactions
        let all_dates = chart_data.get_all_dates();

        // Converting the first date string into a Date type
        // This is the current date that we are checking in the loop
        let mut checking_date =
            NaiveDate::parse_from_str(&chart_data.all_txs[0][0], "%d-%m-%Y").unwrap();

        // The final date where the loop will stop
        let final_date = NaiveDate::parse_from_str(
            &chart_data.all_txs[chart_data.all_txs.len() - 1][0],
            "%d-%m-%Y",
        )
        .unwrap();

        let total_loop = final_date.signed_duration_since(checking_date).num_days() as usize;

        if let Some(val) = loop_remaining {
            if *val == 0 {
                if total_loop > 1 {
                    *loop_remaining = Some(total_loop - 1)
                } else {
                    *loop_remaining = None
                }
            } else if *val - 1 > 0 {
                *loop_remaining = Some(*val - 1)
            } else {
                *loop_remaining = None
            }
        }

        let mut to_loop = loop_remaining.as_mut().map(|val| total_loop - *val);

        // labels of the x axis
        date_labels.push(checking_date.to_string());
        date_labels.push(final_date.to_string());

        // data_num represents which index to check out all the txs and balances data
        // to_add_again will become true in cases where two or more transactions shares the same date.
        // So same date transactions will be combined together for one day

        let mut to_add_again = false;
        let mut data_num = 0;
        loop {
            if all_dates.contains(&checking_date) {
                let current_balances = &chart_data.all_balance[data_num];

                // default next_date in case there is no more next_date
                let mut next_date = NaiveDate::from_ymd_opt(2030, 1, 1).unwrap();

                if chart_data.all_txs.len() > data_num + 1 {
                    next_date =
                        NaiveDate::parse_from_str(&chart_data.all_txs[data_num + 1][0], "%d-%m-%Y")
                            .unwrap();
                }
                // as this is a valid transaction with new changes on a different date, previously saved balances
                // are removed. last balances are used when there are dates with no transactions
                last_balances = Vec::new();

                for method_index in 0..all_tx_methods.len() {
                    // keep track of the highest and the lowest point of the balance
                    let current_balance = current_balances[method_index].parse::<f64>().unwrap();
                    if current_balance > highest_balance {
                        highest_balance = current_balance
                    } else if current_balance < lowest_balance {
                        lowest_balance = current_balance
                    }

                    if to_add_again {
                        // if the next date matches with the current date, we will remove the previous data point
                        // and replace it with the current balance

                        let (position, _balance) = datasets[method_index].pop().unwrap();
                        let to_push = vec![(position, current_balance)];
                        datasets[method_index].extend(to_push);
                        last_balances.push(current_balance);
                    } else {
                        let to_push = vec![(current_axis, current_balance)];

                        if datasets.get(method_index).is_some() {
                            datasets[method_index].extend(to_push);
                        } else {
                            datasets.push(to_push)
                        }

                        last_balances.push(current_balance);
                    }
                }

                if next_date == checking_date {
                    // the axis won't move if the next date is the same.
                    to_add_again = true
                } else {
                    to_add_again = false;
                    current_axis += 1.0;
                    checking_date += Duration::days(1);

                    if let Some(val) = to_loop {
                        if val == 0 {
                            break;
                        }
                        to_loop = Some(val - 1);
                    }
                }

                // successfully checked a transaction, we will check the new index in the next iteration
                data_num += 1;
            } else {
                // as the date does not exist in the transaction list, we will use the last used balance and add a point in the chart
                for method_index in 0..all_tx_methods.len() {
                    let to_push = vec![(current_axis, last_balances[method_index])];
                    datasets[method_index].extend(to_push);
                }
                current_axis += 1.0;
                checking_date += Duration::days(1);
            }

            if checking_date >= final_date + Duration::days(1) {
                break;
            }
        }
    }

    // add a 10% extra value to the highest and the lowest balance
    // so the chart can properly render
    highest_balance += highest_balance * 10.0 / 100.0;
    lowest_balance -= lowest_balance * 10.0 / 100.0;

    let diff = (highest_balance - lowest_balance) / 10.0;

    let mut to_add = lowest_balance;

    // go through the lowest balance and keep adding the difference until the highest point
    let mut labels = vec![lowest_balance.to_string()];
    for _i in 0..10 {
        to_add += diff;
        labels.push(format!("{:.2}", to_add));
    }

    let mut color_list = vec![
        Color::Yellow,
        Color::DarkGray,
        Color::LightBlue,
        Color::Magenta,
        Color::Red,
        Color::Green,
        Color::Blue,
    ];

    let mut final_dataset = vec![];

    // loop through the data that was added for each tx_method  and turn them into chart data
    for i in 0..all_tx_methods.len() {
        if color_list.is_empty() {
            color_list.push(Color::Cyan)
        }
        final_dataset.push(
            Dataset::default()
                .name(&all_tx_methods[i])
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(
                    Style::default()
                        .fg(color_list.pop().unwrap())
                        .bg(Color::Rgb(255, 255, 255)),
                )
                .data(&datasets[i]),
        )
    }

    let chart = Chart::new(final_dataset)
        .block(
            Block::default().style(
                Style::default()
                    .bg(Color::Rgb(255, 255, 255))
                    .fg(Color::Rgb(50, 205, 50)),
            ),
        )
        .style(
            Style::default()
                .bg(Color::Rgb(255, 255, 255))
                .fg(Color::Rgb(50, 205, 50)),
        )
        .x_axis(
            Axis::default()
                .title(Span::styled(
                    "",
                    Style::default()
                        .bg(Color::Rgb(255, 255, 255))
                        .fg(Color::Rgb(50, 205, 50)),
                ))
                .style(
                    Style::default()
                        .bg(Color::Rgb(255, 255, 255))
                        .fg(Color::Rgb(50, 205, 50)),
                )
                .bounds([0.0, current_axis - 1.0])
                .labels(date_labels.iter().cloned().map(Span::from).collect()),
        )
        .y_axis(
            Axis::default()
                .title(Span::styled(
                    "",
                    Style::default()
                        .bg(Color::Rgb(255, 255, 255))
                        .fg(Color::Rgb(50, 205, 50)),
                ))
                .style(
                    Style::default()
                        .bg(Color::Rgb(255, 255, 255))
                        .fg(Color::Rgb(50, 205, 50)),
                )
                .bounds([lowest_balance, highest_balance])
                .labels(labels.iter().cloned().map(Span::from).collect()),
        );

    match current_page {
        // previously added a black block to year and month widget if a value is not selected
        // Now we will turn that black block into green if a value is selected
        ChartTab::Months => {
            month_tab = month_tab.highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Rgb(152, 251, 152)),
            );
        }

        ChartTab::Years => {
            year_tab = year_tab.highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Rgb(152, 251, 152)),
            );
        }
        ChartTab::ModeSelection => {
            mode_selection_tab = mode_selection_tab.highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Rgb(152, 251, 152)),
            );
        }
    }

    if chart_hidden_mode {
        f.render_widget(chart, chunks[0]);
    } else {
        f.render_widget(mode_selection_tab, chunks[0]);

        match mode_selection.index {
            0 => {
                f.render_widget(year_tab, chunks[1]);
                f.render_widget(month_tab, chunks[2]);
                f.render_widget(chart, chunks[3]);
            }
            1 => {
                f.render_widget(year_tab, chunks[1]);
                f.render_widget(chart, chunks[2]);
            }
            2 => {
                f.render_widget(chart, chunks[1]);
            }
            _ => {}
        }
    }
}
