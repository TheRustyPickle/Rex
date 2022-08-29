use crate::chart_page::ChartData;
use crate::db::get_all_tx_methods;
use chrono::{naive::NaiveDate, Duration};
use rusqlite::Connection;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    symbols,
    text::Span,
    widgets::{Axis, Block, Chart, Dataset, GraphType},
    Frame,
};

pub fn chart_ui<B: Backend>(f: &mut Frame<B>, chart_data: ChartData) {
    let size = f.size();

    // divide the terminal into various chunks to draw the interface. This is a vertical chunk
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Percentage(70)].as_ref())
        .split(size);
    let block = Block::default().style(
        Style::default()
            .bg(Color::Rgb(255, 255, 255))
            .fg(Color::Rgb(50, 205, 50)),
    );
    f.render_widget(block, size);

    let conn = Connection::open("data.sqlite").expect("Could not connect to database");
    let all_tx_methods = get_all_tx_methods(&conn);

    let mut datasets: Vec<Vec<(f64, f64)>> = Vec::new();
    let mut last_balances = Vec::new();

    for _i in &all_tx_methods {
        datasets.push(vec![(0.0, 0.0)]);
        last_balances.push(0.0);
    }

    let mut lowest_balance = 0.0;
    let mut highest_balance = 0.0;

    let mut date_labels: Vec<String> = vec![];

    if chart_data.all_txs.len() > 0 {
        let all_dates = chart_data.get_all_dates();

        let mut checking_date =
            NaiveDate::parse_from_str(&chart_data.all_txs[0][0], "%d-%m-%Y").unwrap();
        let final_date = NaiveDate::parse_from_str(
            &chart_data.all_txs[chart_data.all_txs.len() - 1][0],
            "%d-%m-%Y",
        )
        .unwrap();

        date_labels.push(checking_date.to_string());
        date_labels.push(final_date.to_string());

        let mut current_axis = 1.0;

        let mut data_num = 0;
        let mut to_add_again = false;

        loop {
            if all_dates.contains(&checking_date) {
                let current_balances = &chart_data.all_balance[data_num];

                let mut next_date = NaiveDate::from_ymd(2030, 1, 1);

                if chart_data.all_txs.len() > data_num + 1 {
                    next_date =
                        NaiveDate::parse_from_str(&chart_data.all_txs[data_num + 1][0], "%d-%m-%Y")
                            .unwrap();
                }

                let mut method_index = 0;
                last_balances = Vec::new();

                for _method in &all_tx_methods {
                    let cu_bal = current_balances[method_index].parse::<f64>().unwrap();
                    if cu_bal > highest_balance {
                        highest_balance = cu_bal
                    } else if cu_bal < lowest_balance {
                        lowest_balance = cu_bal
                    }

                    if to_add_again == true {
                        let (position, _balance) = datasets[method_index].pop().unwrap();
                        let to_push = vec![(position, cu_bal)];
                        datasets[method_index].extend(to_push);
                        last_balances.push(cu_bal);
                    } else {
                        let to_push = vec![(current_axis, cu_bal)];
                        datasets[method_index].extend(to_push);
                        last_balances.push(cu_bal);
                    }
                    method_index += 1;
                }

                if next_date == checking_date {
                    to_add_again = true
                } else {
                    to_add_again = false;
                    current_axis += 1.0;
                    checking_date += Duration::days(1);
                }

                data_num += 1;
            } else {
                let mut method_index = 0;
                for _method in &all_tx_methods {
                    let to_push = vec![(current_axis, last_balances[method_index])];
                    datasets[method_index].extend(to_push);
                    method_index += 1;
                }
                current_axis += 1.0;
                checking_date += Duration::days(1);
            }

            if checking_date >= final_date + Duration::days(1) {
                break;
            }
        }
    }

    highest_balance += highest_balance * 10.0 / 100.0;
    lowest_balance -= lowest_balance * 10.0 / 100.0;

    let diff = (highest_balance - lowest_balance) / 5.0;

    let mut to_add = lowest_balance;

    let mut labels = vec![lowest_balance.to_string()];
    for _i in 0..5 {
        to_add += diff;
        labels.push(to_add.to_string());
    }

    let mut color_list = vec![Color::Red, Color::Green, Color::Blue, Color::Yellow];

    let mut final_dataset = vec![];

    for i in 0..all_tx_methods.len() {
        final_dataset.push(
            Dataset::default()
                .name(&all_tx_methods[i])
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(color_list.pop().unwrap()))
                .data(&datasets[i]),
        )
    }

    let chart = Chart::new(final_dataset)
        .block(Block::default().title("Chart"))
        .x_axis(
            Axis::default()
                .title(Span::styled("X Axis", Style::default().fg(Color::Red)))
                .style(Style::default().fg(Color::White))
                .bounds([0.0, 50.0])
                .labels(date_labels.iter().cloned().map(Span::from).collect()),
        )
        .y_axis(
            Axis::default()
                .title(Span::styled("Y Axis", Style::default().fg(Color::Red)))
                .style(Style::default().fg(Color::White))
                .bounds([lowest_balance, highest_balance])
                .labels(labels.iter().cloned().map(Span::from).collect()),
        );

    f.render_widget(chart, chunks[0]);
}
