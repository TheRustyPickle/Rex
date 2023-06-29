use crate::chart_page::ChartData;
use crate::page_handler::{ChartTab, IndexedData, BACKGROUND, BOX, SELECTED};
use crate::utility::{create_tab, get_all_tx_methods, main_block};
use chrono::{naive::NaiveDate, Duration};
use ratatui::backend::Backend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Axis, Block, Chart, Dataset, GraphType};
use ratatui::{symbols, Frame};
use rusqlite::Connection;

/// Creates the balance chart from the transactions
#[cfg(not(tarpaulin_include))]
pub fn chart_ui<B: Backend>(
    f: &mut Frame<B>,
    months: &IndexedData,
    years: &IndexedData,
    mode_selection: &IndexedData,
    chart_data: &ChartData,
    current_page: &ChartTab,
    chart_hidden_mode: bool,
    loop_remaining: &mut Option<f64>,
    conn: &Connection,
) {
    let size = f.size();
    let (all_txs, all_balance) = chart_data.get_data(mode_selection, months.index, years.index);

    // divide the terminal into various chunks to draw the interface. This is a vertical chunk
    let mut main_layout = Layout::default().direction(Direction::Vertical).margin(2);

    // don't create any other chunk if hidden mode is enabled. Create 1 chunk that will be used for the chart itself
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

    // creates border around the entire terminal
    f.render_widget(main_block(), size);

    let mut month_tab = create_tab(months, "Months");

    let mut year_tab = create_tab(years, "Years");

    let mut mode_selection_tab = create_tab(mode_selection, "Modes");

    let all_tx_methods = get_all_tx_methods(conn);

    // a vector containing another vector with vec![X, Y] coordinate of where to render chart points
    let mut datasets: Vec<Vec<(f64, f64)>> = Vec::new();
    let mut last_balances = Vec::new();

    // adding default initial value if no data to load
    if all_txs.is_empty() {
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
    if !all_txs.is_empty() {
        // contains all dates of the transactions
        let all_dates = chart_data.get_all_dates(mode_selection, months.index, years.index);

        let mut checking_date = NaiveDate::parse_from_str(&all_txs[0][0], "%d-%m-%Y").unwrap();

        // The final date where the loop will stop
        let final_date =
            NaiveDate::parse_from_str(&all_txs[all_txs.len() - 1][0], "%d-%m-%Y").unwrap();

        // total days = number of loops required to render everything
        let total_loop = final_date.signed_duration_since(checking_date).num_days() as f64;

        // When chart ui is selected, start by rendering this amount of day worth of data,
        // then render_size * 2, 3 and so on until the final day is reached, creating a small animation.
        // Numbers were determined after checking with data filled db and with --release flag
        let render_size = if total_loop > 4000.0 {
            (total_loop * 0.7) / 100.0
        } else if total_loop > 2000.0 {
            (total_loop * 0.5) / 100.0
        } else if total_loop > 1000.0 {
            (total_loop * 0.4) / 100.0
        } else if total_loop > 500.0 {
            (total_loop * 0.3) / 100.0
        } else if total_loop > 360.0 {
            (total_loop * 0.4) / 100.0
        } else if total_loop > 200.0 {
            (total_loop * 0.2) / 100.0
        } else if total_loop < 50.0 {
            (total_loop * 2.0) / 100.0
        } else {
            1.0
        };

        // default value = Some(0). After each loop this value goes down by render_size. Once <=0, turn it into None.
        // When it's None, it won't stop the loop in the middle.
        if let Some(val) = loop_remaining {
            if *val == 0.0 {
                if total_loop > render_size {
                    *loop_remaining = Some(total_loop - render_size)
                } else {
                    *loop_remaining = None
                }
            } else if *val - render_size > 0.0 {
                *loop_remaining = Some(*val - render_size)
            } else {
                *loop_remaining = None
            }
        }

        // total times to loop this time. If None, then render everything
        let mut to_loop = loop_remaining.as_mut().map(|val| total_loop - *val);

        // labels of the x axis
        date_labels.push(checking_date.to_string());
        date_labels.push(final_date.to_string());

        // data_num represents which index to check out from all the txs and balances data.
        // to_add_again will become true in cases where two or more transactions shares the same date simultaneously.
        // Same date transactions movements will be combined together into 1 chart location

        let mut to_add_again = false;
        let mut data_num = 0;
        loop {
            if all_dates.contains(&checking_date) {
                let current_balances = &all_balance[data_num];

                // default next_date in case there is no more next_date
                let mut next_date = NaiveDate::from_ymd_opt(2040, 1, 1).unwrap();

                if all_txs.len() > data_num + 1 {
                    next_date =
                        NaiveDate::parse_from_str(&all_txs[data_num + 1][0], "%d-%m-%Y").unwrap();
                }
                // new valid transactions so the earlier looped balance is not required.
                // if no tx exists in a date, data from last_balances/previous valid date is used to compensate for it
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
                        // if to_add_again is true, means in the last loop, the date and the current date was the same
                        // as the date is the same, the data needs to be merged thus using the same x y point in the chart.
                        // pop the last one added and that to last_balance. If the next date is the same,
                        // last_balance will be used to keep on merging the data.

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

            if !to_add_again {
                if let Some(val) = to_loop {
                    // break the loop if total day amount is reached
                    if val - 1.0 <= 0.0 {
                        date_labels.pop().unwrap();
                        date_labels.push(checking_date.to_string());
                        break;
                    }
                    to_loop = Some(val - 1.0);
                }
            }

            if checking_date >= final_date + Duration::days(1) {
                break;
            }
        }
    } else {
        *loop_remaining = None;
    }
    // add a 10% extra value to the highest and the lowest balance
    // so the chart can properly render
    highest_balance += highest_balance * 10.0 / 100.0;
    lowest_balance -= lowest_balance * 10.0 / 100.0;

    let diff = (highest_balance - lowest_balance) / 10.0;

    let mut to_add = lowest_balance;

    // go through the lowest balance and keep adding the difference until the highest point
    let mut labels = vec![lowest_balance.to_string()];
    // 10 labels, so loop 10 times
    for _i in 0..10 {
        to_add += diff;
        labels.push(format!("{:.2}", to_add));
    }

    let mut color_list = vec![
        Color::LightRed,
        Color::LightBlue,
        Color::LightYellow,
        Color::Gray,
        Color::Black,
        Color::Yellow,
        Color::Green,
        Color::Red,
        Color::Blue,
        Color::Magenta,
    ];

    let mut final_dataset = vec![];

    // loop through the data that was added for each tx_method  and turn them into chart data
    for i in 0..all_tx_methods.len() {
        // run out of colors = cyan default
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
                        .bg(BACKGROUND),
                )
                .data(&datasets[i]),
        )
    }

    let chart = Chart::new(final_dataset)
        .block(Block::default().style(Style::default().bg(BACKGROUND).fg(BOX)))
        .style(Style::default().bg(BACKGROUND).fg(BOX))
        .x_axis(
            Axis::default()
                .title(Span::styled("", Style::default().bg(BACKGROUND).fg(BOX)))
                .style(Style::default().bg(BACKGROUND).fg(BOX))
                .bounds([0.0, current_axis - 1.0])
                .labels(date_labels.iter().cloned().map(Span::from).collect()),
        )
        .y_axis(
            Axis::default()
                .title(Span::styled("", Style::default().bg(BACKGROUND).fg(BOX)))
                .style(Style::default().bg(BACKGROUND).fg(BOX))
                .bounds([lowest_balance, highest_balance])
                .labels(labels.iter().cloned().map(Span::from).collect()),
        );

    match current_page {
        // previously added a black block to year and month widget if a value is not selected
        // Now we will turn that black block into green if a value is selected
        ChartTab::Months => {
            month_tab = month_tab
                .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(SELECTED));
        }

        ChartTab::Years => {
            year_tab = year_tab
                .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(SELECTED));
        }
        ChartTab::ModeSelection => {
            mode_selection_tab = mode_selection_tab
                .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(SELECTED));
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
