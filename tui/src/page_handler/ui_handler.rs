use crossterm::event::{self, Event, KeyEventKind, poll};
use ratatui::Terminal;
use ratatui::backend::Backend;
use rex_app::conn::{DbConn, FetchNature, FullRecurringTx};
use rex_app::ui_helper::DateType;
use rex_app::views::SearchView;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::config::Config;
use crate::key_checker::{
    InputKeyHandler, activity_keys, add_tx_keys, chart_keys, home_keys, initial_keys,
    recurring_keys, search_keys, summary_keys,
};
use crate::outputs::{HandlingOutput, UiHandlingError};
use crate::page_handler::{
    ActivityTab, ChartTab, CurrentUi, HomeTab, IndexedData, SortingType, SummaryTab, TableData,
    TxTab,
};
use crate::pages::{
    InfoPopupState, InitialData, PopupType, activity_ui, add_tx_ui, chart_ui, home_ui, initial_ui,
    recurring_ui, search_ui, summary_ui,
};
use crate::theme::Theme;
use crate::tx_handler::TxData;
use crate::utility::LerpState;

/// Starts the interface and runs the app
pub fn start_app<B: Backend>(
    terminal: &mut Terminal<B>,
    new_version_data: Arc<Mutex<Option<Vec<String>>>>,
    config: &mut Config,
    conn: &mut DbConn,
) -> Result<HandlingOutput, UiHandlingError> {
    // Set up the initial state of every page and widget
    let mut popup_status = PopupType::Nothing;

    let recurring_added = match conn.process_due_recurring_txs() {
        Ok(count) => count,
        Err(e) => {
            let state = InfoPopupState::Error(format!("Failed to process recurring TXs: {e}"));
            popup_status = PopupType::new_info(state);
            0
        }
    };

    let mut theme = Theme::new_index(config.theme_index.unwrap_or(0));

    // Indexed month/year/mode/method lists backing each page's selectors. The
    // `_no_local` variants start at the first entry instead of the device's
    // current month or year
    let mut home_months = IndexedData::new_monthly();
    let mut home_years = IndexedData::new_yearly();
    let mut chart_months = IndexedData::new_monthly_no_local();
    let mut chart_years = IndexedData::new_yearly_no_local();
    let mut chart_modes = IndexedData::new_modes();
    let mut chart_tx_methods = IndexedData::new_tx_methods_cumulative(conn);
    let mut summary_months = IndexedData::new_monthly_no_local();
    let mut summary_years = IndexedData::new_yearly_no_local();
    let mut summary_modes = IndexedData::new_modes();
    let mut activity_months = IndexedData::new_monthly();
    let mut activity_years = IndexedData::new_yearly();

    // The selected widget on the homepage. Default set to the month selection
    let mut home_tab = HomeTab::Months;

    // How summary table will be sorted
    let mut summary_sort = SortingType::Tags;

    // An empty search view, contains relevant data to create the search UI
    let mut search_txs = SearchView::new_empty();

    // Contains the TX views for that month and year. Used for home page data + balances
    let mut home_txs = conn
        .fetch_txs_with_str(
            home_months.get_selected_value(),
            home_years.get_selected_value(),
            FetchNature::Monthly,
        )
        .unwrap();

    // Home TX data but in vector form with index tracking. Used for creating the table in the UI
    let mut home_table = TableData::new(home_txs.tx_array());

    // The page which is currently selected. Default is the initial page
    let mut page = CurrentUi::Initial;

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
    // Store the current selected widget on Recurring page
    let mut recurring_tab = TxTab::Nothing;

    // Holds the data that will be/are inserted into the Add TX page's input fields
    let mut add_tx_data = TxData::new();
    // Holds the data that will be/are inserted into the Search page's input fields
    let mut search_data = TxData::new_empty();
    // Holds the data that will be/are inserted into the Recurring page's input fields
    let mut recurring_data = TxData::new_empty();

    // TXs of the selected period, used to draw the chart
    let mut chart_view = conn
        .get_chart_view_with_str(
            chart_months.get_selected_value(),
            chart_years.get_selected_value(),
            FetchNature::Monthly,
        )
        .unwrap();

    // TXs of the selected period, used to generate the summary
    let mut summary_view = conn
        .get_summary_with_str(
            summary_months.get_selected_value(),
            summary_years.get_selected_value(),
            FetchNature::Monthly,
        )
        .unwrap();

    // TXs of the selected month and year, used to generate the activity table
    let mut activity_view = conn
        .get_activity_view_with_str(
            activity_months.get_selected_value(),
            activity_years.get_selected_value(),
        )
        .unwrap();

    // The generated summary
    let mut full_summary = summary_view.generate_summary(None, conn);

    // Data for the Summary Page's table
    let mut summary_table = TableData::new(summary_view.tags_array(None, conn));

    // Data for the Search Page's table
    let mut search_table = TableData::new(Vec::new());

    // Data for the Activity Page's table
    let mut activity_table = TableData::new(activity_view.get_activity_table());

    // The currently known recurring transaction rules
    let mut recurring_txs = conn.get_recurring_txs().unwrap();

    // Startup screen data, loaded once since that page redraws every few milliseconds
    let initial_data = InitialData::load(conn, &recurring_txs, recurring_added);

    // Data for the Recurring Page's table
    let mut recurring_table = TableData::new(
        recurring_txs
            .iter()
            .map(FullRecurringTx::to_array)
            .collect(),
    );

    // The initial page REX loading index
    let mut starter_index = 0;

    // Whether the chart is in hidden mode
    let mut chart_hidden_mode = false;

    // Whether the chart has hidden legends
    let mut chart_hidden_legends = false;

    // Whether the summary is in hidden mode
    let mut summary_hidden_mode = false;

    // Map of which TX methods are activated in the chart
    let mut chart_activated_methods = conn
        .get_tx_methods_cumulative()
        .into_iter()
        .map(|s| (s, true))
        .collect();

    // The generated balance section on the Add TX UI
    let mut add_tx_balance = Vec::new();

    let mut lerp_state = LerpState::new(1.0);

    let mut version_checked = false;

    // How it works:
    // Every value set up above -> rendered on the matching page -> wait for a key press.
    //
    // As long as a lerp is running the UI keeps re-rendering until all of them end
    //
    // On a key press the mutable values are handed to InputKeyHandler, which mutates
    // them -> the loop restarts -> the new values are sent to the interface -> repeat
    loop {
        if !version_checked {
            let update_lock = new_version_data.lock().unwrap();
            if let Some(data) = update_lock.as_ref() {
                if !data.is_empty() {
                    let state = InfoPopupState::NewUpdate(data.to_vec());
                    popup_status = PopupType::new_info(state);
                }
                version_checked = true;
            }
        }
        // With no TX method at all, keep asking the user to create one
        if conn.is_tx_method_empty()
            && let PopupType::Nothing = popup_status
        {
            popup_status = PopupType::new_choice_config_forced(&theme);
        }

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
                        &mut lerp_state,
                        &mut home_txs,
                        &theme,
                        conn,
                    ),

                    CurrentUi::AddTx => add_tx_ui(
                        f,
                        &mut add_tx_balance,
                        &add_tx_data,
                        &add_tx_tab,
                        &mut lerp_state,
                        &theme,
                        conn,
                    ),

                    CurrentUi::Initial => initial_ui(f, starter_index, &initial_data, &theme),

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
                        &theme,
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
                        &theme,
                        conn,
                    ),
                    CurrentUi::Search => search_ui(
                        f,
                        &search_data,
                        &search_tab,
                        &mut search_table,
                        search_date_type,
                        &mut lerp_state,
                        &theme,
                    ),
                    CurrentUi::Activity => activity_ui(
                        f,
                        &activity_months,
                        &activity_years,
                        &activity_tab,
                        &activity_view,
                        &mut activity_table,
                        &mut lerp_state,
                        &theme,
                    ),
                    CurrentUi::Recurring => recurring_ui(
                        f,
                        &recurring_data,
                        &recurring_tab,
                        &mut recurring_table,
                        &mut lerp_state,
                        &theme,
                    ),
                }

                popup_status.show_ui(f, &theme);
            })
            .map_err(|e| {
                let e = e.to_string();
                UiHandlingError::Drawing(e)
            })?;

        // Based on the UI status, either start polling for key press or continue the loop
        match page {
            CurrentUi::Initial => {
                // Initial page will loop to animate the text
                if !poll(Duration::from_millis(40)).map_err(UiHandlingError::Polling)? {
                    starter_index = (starter_index + 1) % 27;
                    continue;
                }
            }
            CurrentUi::Home
            | CurrentUi::AddTx
            | CurrentUi::Summary
            | CurrentUi::Search
            | CurrentUi::Chart
            | CurrentUi::Activity
            | CurrentUi::Recurring => {
                // If at least 1 lerp is in progress and no key press detected, continue the loop
                if lerp_state.has_active_lerps()
                    && !poll(Duration::from_millis(2)).map_err(UiHandlingError::Polling)?
                {
                    continue;
                }
            }
        }

        // Wait for key press
        if let Event::Key(key) = event::read().map_err(UiHandlingError::Polling)? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            let mut handler = InputKeyHandler::new(
                key,
                &mut page,
                &mut add_tx_balance,
                &mut popup_status,
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
                &mut recurring_data,
                &mut recurring_tab,
                &mut recurring_table,
                &mut recurring_txs,
                &mut chart_hidden_mode,
                &mut chart_hidden_legends,
                &mut summary_hidden_mode,
                &mut chart_activated_methods,
                &mut lerp_state,
                config,
                &mut theme,
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
                CurrentUi::Recurring => recurring_keys(&mut handler),
            };

            match status {
                Ok(output) => {
                    if let Some(output) = output {
                        return Ok(output);
                    }
                }
                Err(e) => {
                    let state = InfoPopupState::Error(e.to_string());
                    popup_status = PopupType::new_info(state);
                }
            }
        }
    }
}
