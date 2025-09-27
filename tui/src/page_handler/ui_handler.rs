use app::conn::{DbConn, FetchNature};
use app::ui_helper::DateType;
use app::views::SearchView;
use crossterm::event::poll;
use crossterm::event::{self, Event, KeyEventKind};
use ratatui::Terminal;
use ratatui::backend::Backend;
use ratatui::layout::Constraint;
use ratatui::style::Color;
use std::time::Duration;

use crate::key_checker::{
    InputKeyHandler, activity_keys, add_tx_keys, chart_keys, home_keys, initial_keys, search_keys,
    summary_keys,
};
use crate::outputs::{HandlingOutput, UiHandlingError};
use crate::page_handler::{
    ActivityTab, ChartTab, CurrentUi, DeletionStatus, HomeTab, IndexedData, PopupState,
    SortingType, SummaryTab, TableData, TxTab,
};
use crate::pages::{
    PopupData, activity_ui, add_tx_ui, chart_ui, home_ui, initial_ui, search_ui, summary_ui,
};
use crate::tx_handler::TxData;
use crate::utility::LerpState;

pub const BACKGROUND: Color = Color::Rgb(245, 245, 255);
pub const TEXT: Color = Color::Rgb(153, 78, 236);
pub const BOX: Color = Color::Rgb(255, 87, 51);
pub const SELECTED: Color = Color::Rgb(151, 251, 151);
pub const HIGHLIGHTED: Color = Color::Rgb(38, 38, 38);
pub const HEADER: Color = Color::Rgb(0, 150, 255);
pub const RED: Color = Color::Rgb(255, 51, 51);
pub const BLUE: Color = Color::Rgb(51, 51, 255);
pub const GRAY: Color = Color::Rgb(128, 128, 128);

/// Starts the interface and run the app
pub fn start_app<B: Backend>(
    terminal: &mut Terminal<B>,
    new_version_data: &Option<Vec<String>>,
    conn: &mut DbConn,
) -> Result<HandlingOutput, UiHandlingError> {
    // Setting up some default values. Let's go through all of them

    // Contains the homepage month list that is indexed

    let mut home_months = IndexedData::new_monthly();
    // Contains the homepage year list that is indexed
    let mut home_years = IndexedData::new_yearly();
    // Contains the chart page month list that is indexed
    let mut chart_months = IndexedData::new_monthly_no_local();
    // Contains the chart page year list that is indexed
    let mut chart_years = IndexedData::new_yearly_no_local();
    // Contains the chart page mode selection list that is indexed
    let mut chart_modes = IndexedData::new_modes();
    // Contains the chart page tx method selection list that is indexed
    let mut chart_tx_methods = IndexedData::new_tx_methods_cumulative(conn);

    // Contains the summary page month list that is indexed
    let mut summary_months = IndexedData::new_monthly_no_local();
    // Contains the summary page year list that is indexed
    let mut summary_years = IndexedData::new_yearly_no_local();
    // Contains the summary page mode selection list that is indexed
    let mut summary_modes = IndexedData::new_modes();
    // Contains the Activity page month list that is indexed
    let mut activity_years = IndexedData::new_yearly();
    // Contains the Activity page month list that is indexed
    let mut activity_months = IndexedData::new_monthly();

    // The selected widget on the HomePage. Default set to the month selection
    let mut home_tab = HomeTab::Months;

    // How summary table will be sorted
    let mut summary_sort = SortingType::Tags;

    // Stores all activity for a specific month of a year alongside the txs involved in an activity

    let mut search_txs = SearchView::new_empty();

    // Contains the tx views for that month and year. Used for home page data + balances
    let mut home_txs = conn
        .fetch_txs_with_str(
            home_months.get_selected_value(),
            home_years.get_selected_value(),
            FetchNature::Monthly,
        )
        .unwrap();

    // Home tx data but in vector form with index tracking
    let mut home_table = TableData::new(home_txs.tx_array());

    // The page which is currently selected. Default is the initial page
    let mut page = CurrentUi::Initial;
    // stores current popup status
    let mut popup_state = if let Some(data) = new_version_data {
        PopupState::NewUpdate(data.to_owned())
    } else {
        PopupState::Nothing
    };

    // Stores the current selected widget on Add Transaction page
    let mut add_tx_tab = TxTab::Nothing;
    // Store the current selected widget on Chart page
    let mut chart_tab = ChartTab::ModeSelection;
    // Store the current selected widget on Summary page
    let mut summary_tab = SummaryTab::ModeSelection;
    // Store the current selected widget on Search page
    let mut search_tab = TxTab::Nothing;
    // Store the current searching date type
    let mut search_date_type = DateType::Exact;
    // Store the current selected widget on Activity page
    let mut activity_tab = ActivityTab::Years;

    // Holds the data that will be/are inserted into the Add Tx page's input fields
    let mut add_tx_data = TxData::new();
    // Holds the data that will be/are inserted into the Search page's input fields
    let mut search_data = TxData::new_empty();
    let mut chart_view = conn
        .get_chart_view_with_str(
            chart_months.get_selected_value(),
            chart_years.get_selected_value(),
            FetchNature::Monthly,
        )
        .unwrap();
    // Holds the popup data that will be/are inserted into the Popup page
    let mut popup_data = PopupData::new();

    let mut summary_view = conn
        .get_summary_with_str(
            summary_months.get_selected_value(),
            summary_years.get_selected_value(),
            FetchNature::Monthly,
        )
        .unwrap();

    let mut activity_view = conn
        .get_activity_view_with_str(
            activity_months.get_selected_value(),
            activity_years.get_selected_value(),
        )
        .unwrap();

    let mut full_summary = summary_view.generate_summary(None, conn);

    // Data for the Summary Page's table
    let mut summary_table = TableData::new(summary_view.tags_array(None, conn));

    // Data for the Search Page's table
    let mut search_table = TableData::new(Vec::new());

    // Data for the Activity Page's table
    let mut activity_table = TableData::new(activity_view.get_activity_table());

    // The initial page REX loading index
    let mut starter_index = 0;

    // Whether the chart is in hidden mode
    let mut chart_hidden_mode = false;

    // Whether the chart has hidden legends
    let mut chart_hidden_legends = false;

    // Whether the summary is in hidden mode
    let mut summary_hidden_mode = false;

    // The initial popup when deleting tx will start on Yes value
    let mut deletion_status: DeletionStatus = DeletionStatus::Yes;

    // Contains whether in the chart whether a tx method is activated or not
    let mut chart_activated_methods = conn
        .get_tx_methods_cumulative()
        .into_iter()
        .map(|s| (s, true))
        .collect();

    let mut popup_scroll_position = 0;
    let mut max_popup_scroll = 0;

    // Home and Add TX Page balance data
    let mut add_tx_balance = Vec::new();
    // Home and add TX page balance section's column space
    let mut width_data = Vec::new();
    let total_columns = conn.get_tx_methods().len() + 2;
    let width_percent = (100 / total_columns) as u16;

    // Save the % of space each column should take in the Balance section based on the total
    // transaction methods/columns available
    for _ in 0..total_columns {
        width_data.push(Constraint::Percentage(width_percent));
    }

    let mut lerp_state = LerpState::new(1.0);

    // How it work:
    // Default value from above -> Goes to an interface page and render -> Wait for an event key press.
    //
    // If no key press is detected in certain position it will start the next iteration -> interface -> Key check
    // Otherwise it will poll for key press and locks the position
    //
    // If key press is detected, send most of the &mut values to InputKeyHandler -> Gets mutated based on key press
    // -> loop ends -> start from beginning -> Send the new mutated values to the interface -> Keep up
    loop {
        // Passing out relevant data to the UI function
        terminal
            .draw(|f| {
                match page {
                    CurrentUi::Home => home_ui(
                        f,
                        &home_months,
                        &home_years,
                        &mut home_table,
                        &home_tab,
                        &mut width_data,
                        &mut lerp_state,
                        &mut home_txs,
                        conn,
                    ),

                    CurrentUi::AddTx => add_tx_ui(
                        f,
                        &mut add_tx_balance,
                        &add_tx_data,
                        &add_tx_tab,
                        &mut width_data,
                        &mut lerp_state,
                        conn,
                    ),

                    CurrentUi::Initial => initial_ui(f, starter_index),

                    CurrentUi::Chart => chart_ui(
                        f,
                        &chart_months,
                        &chart_years,
                        &chart_modes,
                        &chart_tx_methods,
                        &chart_tab,
                        chart_hidden_mode,
                        chart_hidden_legends,
                        &chart_activated_methods,
                        &mut lerp_state,
                        &chart_view,
                        conn,
                    ),

                    CurrentUi::Summary => summary_ui(
                        f,
                        &summary_months,
                        &summary_years,
                        &summary_modes,
                        &mut summary_table,
                        &summary_tab,
                        summary_hidden_mode,
                        &summary_sort,
                        &mut lerp_state,
                        &full_summary,
                        conn,
                    ),
                    CurrentUi::Search => search_ui(
                        f,
                        &search_data,
                        &search_tab,
                        &mut search_table,
                        &search_date_type,
                        &mut lerp_state,
                    ),
                    CurrentUi::Activity => activity_ui(
                        f,
                        &activity_months,
                        &activity_years,
                        &activity_tab,
                        &activity_view,
                        &mut activity_table,
                        &mut lerp_state,
                    ),
                }
                popup_data.create_popup(
                    f,
                    &popup_state,
                    &deletion_status,
                    popup_scroll_position,
                    &mut max_popup_scroll,
                );
            })
            .map_err(UiHandlingError::DrawingError)?;

        // Based on the UI status, either start polling for key press or continue the loop
        match page {
            CurrentUi::Initial => {
                // Initial page will loop indefinitely to animate the text
                if !poll(Duration::from_millis(40)).map_err(UiHandlingError::PollingError)? {
                    starter_index = (starter_index + 1) % 27;
                    continue;
                }
            }
            CurrentUi::Home
            | CurrentUi::AddTx
            | CurrentUi::Summary
            | CurrentUi::Search
            | CurrentUi::Chart
            | CurrentUi::Activity => {
                // If at least 1 lerp is in progress and no key press detected, continue the loop
                if lerp_state.has_active_lerps()
                    && !poll(Duration::from_millis(2)).map_err(UiHandlingError::PollingError)?
                {
                    continue;
                }
            }
        }

        // If not inside one of the duration polling, wait for key press
        if let Event::Key(key) = event::read().map_err(UiHandlingError::PollingError)? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            let mut handler = InputKeyHandler::new(
                key,
                &mut page,
                &mut add_tx_balance,
                &mut popup_state,
                &mut add_tx_tab,
                &mut chart_tab,
                &mut summary_tab,
                &mut home_tab,
                &mut add_tx_data,
                &mut chart_view,
                &mut summary_view,
                &mut full_summary,
                &mut home_table,
                &mut home_txs,
                &mut summary_table,
                &mut home_months,
                &mut home_years,
                &mut chart_months,
                &mut chart_years,
                &mut chart_modes,
                &mut chart_tx_methods,
                &mut summary_months,
                &mut summary_years,
                &mut summary_modes,
                &mut summary_sort,
                &mut search_data,
                &mut search_date_type,
                &mut search_tab,
                &mut search_table,
                &mut search_txs,
                &mut activity_months,
                &mut activity_years,
                &mut activity_tab,
                &mut activity_view,
                &mut activity_table,
                &mut chart_hidden_mode,
                &mut chart_hidden_legends,
                &mut summary_hidden_mode,
                &mut deletion_status,
                &mut chart_activated_methods,
                &mut popup_scroll_position,
                &mut max_popup_scroll,
                &mut lerp_state,
                conn,
            );

            let status = match handler.page {
                CurrentUi::Initial => initial_keys(&mut handler),
                CurrentUi::Home => home_keys(&mut handler),
                CurrentUi::AddTx => add_tx_keys(&mut handler),
                CurrentUi::Chart => chart_keys(&mut handler),
                CurrentUi::Summary => summary_keys(&mut handler),
                CurrentUi::Search => search_keys(&mut handler),
                CurrentUi::Activity => activity_keys(&mut handler),
            };

            // If there is a status it means it needs to be handled outside the UI
            // Example quitting or J press for user inputs
            if let Some(output) = status {
                return Ok(output);
            }
        }
    }
}
