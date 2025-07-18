use chrono::{naive::NaiveDate, Duration};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols::Marker;
use ratatui::text::Span;
use ratatui::widgets::{Axis, Block, Chart, Dataset, GraphType};
use ratatui::Frame;
use rusqlite::Connection;
use std::collections::HashMap;

use crate::chart_page::ChartData;
use crate::page_handler::{ChartTab, IndexedData, BACKGROUND, BOX, SELECTED};
use crate::utility::{
    create_tab, create_tab_activation, get_all_tx_methods, get_all_tx_methods_cumulative,
    main_block, LerpState,
};

/// Creates the balance chart from the transactions
#[cfg(not(tarpaulin_include))]
pub fn chart_ui<S: ::std::hash::BuildHasher>(
    f: &mut Frame,
    months: &IndexedData,
    years: &IndexedData,
    mode_selection: &IndexedData,
    chart_tx_methods: &IndexedData,
    chart_data: &ChartData,
    current_page: &ChartTab,
    chart_hidden_mode: bool,
    chart_hidden_legends: bool,
    chart_activated_methods: &HashMap<String, bool, S>,
    lerp_state: &mut LerpState,
    conn: &Connection,
) {
    let size = f.area();
    let (all_txs, all_balance) = chart_data.get_data(mode_selection, months.index, years.index);

    // Divide the terminal into various chunks to draw the interface. This is a vertical chunk
    let mut main_layout = Layout::default().direction(Direction::Vertical).margin(2);

    // Don't create any other chunk if hidden mode is enabled. Create 1 chunk that will be used for the chart itself
    if chart_hidden_mode {
        main_layout = main_layout.constraints([Constraint::Min(0)]);
    } else {
        match mode_selection.index {
            0 => {
                main_layout = main_layout.constraints([
                    // Modes
                    Constraint::Length(3),
                    // Years
                    Constraint::Length(3),
                    // Months
                    Constraint::Length(3),
                    // Tx Method
                    Constraint::Length(3),
                    // Chart
                    Constraint::Min(0),
                ]);
            }
            1 => {
                main_layout = main_layout.constraints([
                    // Modes
                    Constraint::Length(3),
                    // Years
                    Constraint::Length(3),
                    // Tx method
                    Constraint::Length(3),
                    // Chart
                    Constraint::Min(0),
                ]);
            }
            2 => {
                main_layout = main_layout.constraints([
                    // Modes
                    Constraint::Length(3),
                    // Tx method
                    Constraint::Length(3),
                    // Chart
                    Constraint::Min(0),
                ]);
            }
            _ => {}
        }
    }

    let chunks = main_layout.split(size);

    // Creates border around the entire terminal
    f.render_widget(main_block(), size);

    let mut month_tab = create_tab(months, "Months");

    let mut year_tab = create_tab(years, "Years");

    let mut mode_selection_tab = create_tab(mode_selection, "Modes");

    let mut tx_method_selection_tab = create_tab_activation(
        chart_tx_methods,
        "Tx Method Selection",
        chart_activated_methods,
    );

    let all_tx_methods = get_all_tx_methods(conn);
    let all_tx_methods_cumulative = get_all_tx_methods_cumulative(conn);

    // A vector containing another vector vec![X, Y] with coordinate of where to render chart points
    let mut datasets: Vec<Vec<(f64, f64)>> = Vec::new();
    let mut last_balances = Vec::new();

    // Adding default initial value if no data to load
    if all_txs.is_empty() {
        for _i in &all_tx_methods_cumulative {
            datasets.push(vec![(0.0, 0.0)]);
            last_balances.push(0.0);
        }
    }

    let mut lowest_balance = 0.0;
    let mut highest_balance = 0.0;

    let mut date_labels: Vec<String> = vec![];

    let mut current_axis = 0.0;

    // If there are no transactions, we will create an empty chart
    if !all_txs.is_empty() {
        // Contains all dates of the transactions
        let all_dates = chart_data.get_all_dates(mode_selection, months.index, years.index);

        let mut checking_date = NaiveDate::parse_from_str(&all_txs[0][0], "%d-%m-%Y").unwrap();

        // The final date where the loop will stop
        let final_date =
            NaiveDate::parse_from_str(&all_txs[all_txs.len() - 1][0], "%d-%m-%Y").unwrap();

        // Total days = number of loops required to render everything
        let total_loop = final_date.signed_duration_since(checking_date).num_days() as f64;

        // When chart UI is selected, start by rendering this amount of day worth of data,
        // then render_size * 2, 3 and so on until the final day is reached, creating a small animation.
        // Numbers were determined after checking with data filled db and with --release flag
        let lerp_id = "chart_loop_size";
        let mut to_loop = lerp_state.lerp(lerp_id, total_loop);

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

                // Default next_date in case there is no more next_date
                let mut next_date = NaiveDate::from_ymd_opt(2040, 1, 1).unwrap();

                if all_txs.len() > data_num + 1 {
                    next_date =
                        NaiveDate::parse_from_str(&all_txs[data_num + 1][0], "%d-%m-%Y").unwrap();
                }
                // New valid transactions so the earlier looped balance is not required.
                // If no tx exists in a date, data from last_balances/previous valid date is used to compensate for it
                last_balances = Vec::new();

                let mut cumulative_balance = 0.0;

                for method_index in 0..all_tx_methods_cumulative.len() {
                    // Keep track of the highest and the lowest point of the balance
                    let current_balance = if method_index == all_tx_methods.len() {
                        cumulative_balance
                    } else {
                        let balance = current_balances[method_index].parse::<f64>().unwrap();
                        cumulative_balance += balance;
                        balance
                    };

                    // We will not consider the highest/lowest balance if the method is currently deactivated on chart.
                    // We can't directly skip it because the dataset vector expects something in the index of this method.
                    // We will have the data but it just won't be shown on the chart.
                    if chart_activated_methods[&all_tx_methods_cumulative[method_index]] {
                        if current_balance > highest_balance {
                            highest_balance = current_balance;
                        } else if current_balance < lowest_balance {
                            lowest_balance = current_balance;
                        }
                    }

                    if to_add_again {
                        // If to_add_again is true, means in the last loop, the date, and the current date was the same
                        // as the date is the same, the data needs to be merged thus using the same x y point in the chart.
                        // Pop the last one added and that to last_balance. If the next date is the same,
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
                            datasets.push(to_push);
                        }

                        last_balances.push(current_balance);
                    }
                }

                if next_date == checking_date {
                    // The axis won't move if the next date is the same.
                    to_add_again = true;
                } else {
                    to_add_again = false;
                    current_axis += 1.0;
                    checking_date += Duration::days(1);
                }

                // Successfully checked a transaction, we will check the new index in the next iteration
                data_num += 1;
            } else {
                // As the date does not exist in the transaction list, we will use the last used balance and add a point in the chart
                for method_index in 0..all_tx_methods.len() {
                    let to_push = vec![(current_axis, last_balances[method_index])];
                    datasets[method_index].extend(to_push);
                }
                current_axis += 1.0;
                checking_date += Duration::days(1);
            }

            if !to_add_again {
                // Break the loop if total day amount is reached
                if to_loop - 1.0 <= 0.0 {
                    date_labels.pop().unwrap();
                    date_labels.push(checking_date.to_string());
                    break;
                }
                to_loop += -1.0;
            }

            if checking_date >= final_date + Duration::days(1) {
                break;
            }
        }
    }
    // Add a 10% extra value to the highest and the lowest balance
    // so the chart can properly render
    highest_balance += highest_balance * 5.0 / 100.0;
    lowest_balance -= lowest_balance * 5.0 / 100.0;

    let diff = (highest_balance - lowest_balance) / 10.0;

    let mut to_add = lowest_balance;

    // Go through the lowest balance and keep adding the difference until the highest point
    let mut labels = vec![lowest_balance.to_string()];
    // 10 labels, so loop 10 times
    for _i in 0..10 {
        to_add += diff;
        labels.push(format!("{to_add:.2}"));
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

    // Loop through the data that was added for each tx_method and turn them into chart data
    for i in 0..all_tx_methods_cumulative.len() {
        // Run out of colors = cyan default
        if color_list.is_empty() {
            color_list.push(Color::Cyan);
        }

        if !chart_activated_methods[&all_tx_methods_cumulative[i]] {
            continue;
        }

        let mut dataset = Dataset::default()
            .marker(Marker::Braille)
            .graph_type(GraphType::Line)
            .style(
                Style::default()
                    .fg(color_list.pop().unwrap())
                    .bg(BACKGROUND),
            )
            .data(&datasets[i]);

        if !chart_hidden_legends {
            dataset = dataset.name(all_tx_methods_cumulative[i].clone());
        }

        final_dataset.push(dataset);
    }

    let chart = Chart::new(final_dataset)
        .block(Block::default().style(Style::default().bg(BACKGROUND).fg(BOX)))
        .style(Style::default().bg(BACKGROUND).fg(BOX))
        .x_axis(
            Axis::default()
                .title(Span::styled("", Style::default().bg(BACKGROUND).fg(BOX)))
                .style(Style::default().bg(BACKGROUND).fg(BOX))
                .bounds([0.0, current_axis - 1.0])
                .labels(
                    date_labels
                        .iter()
                        .cloned()
                        .map(Span::from)
                        .collect::<Vec<_>>(),
                ),
        )
        .y_axis(
            Axis::default()
                .title(Span::styled("", Style::default().bg(BACKGROUND).fg(BOX)))
                .style(Style::default().bg(BACKGROUND).fg(BOX))
                .bounds([lowest_balance, highest_balance])
                .labels(labels.iter().cloned().map(Span::from).collect::<Vec<_>>()),
        );

    match current_page {
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
        ChartTab::TxMethods => {
            tx_method_selection_tab = tx_method_selection_tab
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
                f.render_widget(tx_method_selection_tab, chunks[3]);
                f.render_widget(chart, chunks[4]);
            }
            1 => {
                f.render_widget(year_tab, chunks[1]);
                f.render_widget(tx_method_selection_tab, chunks[2]);
                f.render_widget(chart, chunks[3]);
            }
            2 => {
                f.render_widget(tx_method_selection_tab, chunks[1]);
                f.render_widget(chart, chunks[2]);
            }
            _ => {}
        }
    }
}
