use tui::{
    symbols,
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Span},
    widgets::{Block, Chart, Dataset, Axis, GraphType},
    Frame,
};
use crate::chart_page::ChartData;
use rusqlite::Connection;
use crate::db::get_all_tx_methods;
use chrono::{Duration, naive::NaiveDate};

pub fn chart_ui<B: Backend>(f: &mut Frame<B>, chart_data: ChartData) {
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

    let conn = Connection::open("data.sqlite").expect("Could not connect to database");
    let all_tx_methods = get_all_tx_methods(&conn);

    let mut another_dataset: Vec<Vec<(f64, f64)>> = Vec::new();
    let mut last_balances = Vec::new();

    for _i in &all_tx_methods {
        another_dataset.push(vec![(0.0, 0.0)]);
        last_balances.push(0.0);
    }

    let all_dates = chart_data.get_all_dates();
    //println!("{chart_date}");
    let mut last_date = NaiveDate::from_ymd(2022, 1, 1);
    let mut last_chart_position = 0.0;

    let mut data_num = 0;
    let mut to_add_again = false;
    loop {
        if all_dates.contains(&last_date) {
            
            let current_balances = &chart_data.all_balance[data_num];

            let mut next_date = NaiveDate::from_ymd(2030, 1, 1);

            if chart_data.all_txs.len() > data_num + 1 {
                next_date = NaiveDate::parse_from_str(&chart_data.all_txs[data_num+1][0], "%d-%m-%Y").unwrap();
            }
            else {
                to_add_again = false;
            }

            let mut method_index = 0;
            last_balances = Vec::new();

            for _method in &all_tx_methods {

                if to_add_again == true {
                    let (position, balance) = another_dataset[method_index].pop().unwrap();
                    let to_push = vec![(position, balance + current_balances[method_index].parse::<f64>().unwrap())];
                    another_dataset[method_index].extend(to_push);
                    last_balances.push(balance + current_balances[method_index].parse::<f64>().unwrap());
                }
                else {
                    let to_push = vec![(last_chart_position, current_balances[method_index].parse::<f64>().unwrap())];
                    another_dataset[method_index].extend(to_push);
                    last_balances.push(current_balances[method_index].parse::<f64>().unwrap());
                }
                method_index += 1;
            }

            if next_date == last_date {
                to_add_again = true
            }
            else {
                to_add_again = false;
                last_chart_position += 5.0;
                last_date += Duration::days(1);
            }
            
            data_num += 1;

            
        } else {
            let mut method_index = 0;
            for _method in &all_tx_methods {
                let to_push = vec![(last_chart_position, last_balances[method_index])];
                another_dataset[method_index].extend(to_push);
                method_index += 1;
                
            }
            last_chart_position += 5.0;
            last_date += Duration::days(1);
        }
        
        if last_date >= NaiveDate::from_ymd(2023, 1, 1) { break }
        
    }

    let mut color_list = vec![Color::Red, Color::Green, Color::Blue, Color::Yellow];

    let mut final_dataset = vec![];
    //println!("{:?}", another_dataset[0]);

    for i in 0..all_tx_methods.len() {
        final_dataset.push(
            Dataset::default()
            .name(&all_tx_methods[i])
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(color_list.pop().unwrap()))
            .data(&another_dataset[i])
        )
    }

    let chart = Chart::new(final_dataset)
        .block(Block::default().title("Chart"))
        .x_axis(Axis::default()
            .title(Span::styled("X Axis", Style::default().fg(Color::Red)))
            .style(Style::default().fg(Color::White))
            .bounds([0.0, 4.0])
            .labels(["0.0", "20.0", "40.0","60.0", "80.0", "100.0"].iter().cloned().map(Span::from).collect()))
        .y_axis(Axis::default()
            .title(Span::styled("Y Axis", Style::default().fg(Color::Red)))
            .style(Style::default().fg(Color::White))
            .bounds([0.0, 500000.0])
            .labels(["0.0", "20000.0", "40000.0","60000.0", "80000.0", "100000.0"].iter().cloned().map(Span::from).collect()));

    f.render_widget(chart, chunks[0]);
}