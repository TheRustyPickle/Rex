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

/// Creates the balance chart from all the transactions in a given year
pub fn chart_ui<B: Backend>(f: &mut Frame<B>, chart_data: ChartData) {
    let size = f.size();

    // divide the terminal into various chunks to draw the interface. This is a vertical chunk
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(size);

    let block = Block::default().style(
        Style::default()
            .bg(Color::Rgb(255, 255, 255))
            .fg(Color::Rgb(50, 205, 50)),
    );
    f.render_widget(block, size);

    // connect to the database and gather all the tx methods
    let conn = Connection::open("data.sqlite").expect("Could not connect to database");
    let all_tx_methods = get_all_tx_methods(&conn);

    let mut datasets: Vec<Vec<(f64, f64)>> = Vec::new();
    let mut last_balances = Vec::new();

    // adding default initial value
    for _i in &all_tx_methods {
        datasets.push(vec![(0.0, 0.0)]);
        last_balances.push(0.0);
    }

    let mut lowest_balance = 0.0;
    let mut highest_balance = 0.0;

    let mut date_labels: Vec<String> = vec![];

    let mut current_axis = 1.0;

    // if there are no transactions, we will create an empty chart
    if chart_data.all_txs.len() > 0 {
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

        // labels of the x axis
        date_labels.push(checking_date.to_string());

        let date_difference = (final_date - checking_date).num_days() / 5;

        let mut starting_point = checking_date;

        for _i in 0..3 {
            starting_point = starting_point + Duration::days(date_difference);
            date_labels.push(starting_point.to_string())
        }
        date_labels.push(final_date.to_string());

        // data_num represents which index to check out all all the txs and balances data
        // to_add_again will become true in cases where two or more transactions shares the same date.
        // So same date transactions will be combined together for one day
        
        let mut to_add_again = false;
        let mut data_num = 0;
        loop {
            if all_dates.contains(&checking_date) {
                let current_balances = &chart_data.all_balance[data_num];

                // default next_date in case there is no more next_date
                let mut next_date = NaiveDate::from_ymd(2030, 1, 1);

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
                    let cu_bal = current_balances[method_index].parse::<f64>().unwrap();
                    if cu_bal > highest_balance {
                        highest_balance = cu_bal
                    } else if cu_bal < lowest_balance {
                        lowest_balance = cu_bal
                    }

                    if to_add_again == true {
                        // if the next date matches with the current date, we will remove the previous data point
                        // and replace it with the current balance

                        let (position, _balance) = datasets[method_index].pop().unwrap();
                        let to_push = vec![(position, cu_bal)];
                        datasets[method_index].extend(to_push);
                        last_balances.push(cu_bal);
                    } else {
                        let to_push = vec![(current_axis, cu_bal)];
                        datasets[method_index].extend(to_push);
                        last_balances.push(cu_bal);
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
        if color_list.len() == 0 {
            color_list.push(Color::Cyan)
        }
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
                .title(Span::styled(
                    "X Axis",
                    Style::default().fg(Color::LightGreen),
                ))
                .style(Style::default().fg(Color::White))
                .bounds([0.0, current_axis])
                .labels(date_labels.iter().cloned().map(Span::from).collect()),
        )
        .y_axis(
            Axis::default()
                .title(Span::styled(
                    "Y Axis",
                    Style::default().fg(Color::LightGreen),
                ))
                .style(Style::default().fg(Color::White))
                .bounds([lowest_balance, highest_balance])
                .labels(labels.iter().cloned().map(Span::from).collect()),
        );

    f.render_widget(chart, chunks[0]);
}
