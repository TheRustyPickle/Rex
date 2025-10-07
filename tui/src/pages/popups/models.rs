use ratatui::Frame;
use ratatui::style::Color;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, FromRepr};

use crate::page_handler::{BLUE, RED, TableData};

pub struct InfoPopup {
    pub scroll_position: usize,
    pub max_scroll: usize,
    pub showing: InfoPopupState,
}

impl InfoPopup {
    pub fn is_new_update(&self) -> bool {
        let new_update = matches!(self.showing, InfoPopupState::NewUpdate(_));
        new_update
    }
}

pub struct ChoicePopup {
    pub table: TableData,
    pub choices: Vec<ChoiceDetails>,
}

pub struct ChoiceDetails {
    pub text: String,
    pub color: Color,
}

#[derive(EnumIter, Display, FromRepr, Copy, Clone)]
pub enum DeletionChoices {
    #[strum(to_string = "Yes")]
    Yes,
    #[strum(to_string = "No")]
    No,
}

impl DeletionChoices {
    fn to_choice(self) -> ChoiceDetails {
        match self {
            Self::Yes => ChoiceDetails {
                text: self.to_string(),
                color: RED,
            },
            Self::No => ChoiceDetails {
                text: self.to_string(),
                color: BLUE,
            },
        }
    }
}

pub enum PopupType {
    Info(InfoPopup),
    Choice(ChoicePopup),
    Nothing,
}

/// Indicates which pop up is currently on and is being shown in the screen
#[derive(Clone)]
pub enum InfoPopupState {
    NewUpdate(Vec<String>),
    HomeHelp,
    AddTxHelp,
    ChartHelp,
    SummaryHelp,
    SearchHelp,
    ActivityHelp,
    ChoiceHelp,
    Error(String),
    ShowDetails(String),
}

impl PopupType {
    pub fn show_ui(&mut self, f: &mut Frame) {
        match self {
            PopupType::Info(info) => info.show_ui(f),
            PopupType::Choice(choice) => choice.show_ui(f),
            PopupType::Nothing => {}
        }
    }

    pub fn new_info(state: InfoPopupState) -> Self {
        PopupType::Info(InfoPopup {
            showing: state,
            scroll_position: 0,
            max_scroll: 0,
        })
    }

    pub fn next(&mut self) {
        match self {
            PopupType::Info(info) => {
                if info.max_scroll == 0 {
                    return;
                }

                info.scroll_position += 1;

                if info.scroll_position > info.max_scroll {
                    info.scroll_position = 0;
                }
            }
            PopupType::Choice(choice) => {
                choice.table.next();
            }
            PopupType::Nothing => {}
        }
    }

    pub fn previous(&mut self) {
        match self {
            PopupType::Info(info) => {
                if info.max_scroll == 0 {
                    return;
                }

                if info.scroll_position == 0 {
                    info.scroll_position = info.max_scroll;
                } else {
                    info.scroll_position -= 1;
                }
            }
            PopupType::Choice(choice) => {
                choice.table.previous();
            }
            PopupType::Nothing => {}
        }
    }

    pub fn new_choice_deletion() -> Self {
        let choices: Vec<ChoiceDetails> = DeletionChoices::iter()
            .map(DeletionChoices::to_choice)
            .collect();

        let table_items = choices.iter().map(|c| vec![c.text.clone()]).collect();
        let mut table_data = TableData::new(table_items);
        table_data.state.select(Some(0));

        PopupType::Choice(ChoicePopup {
            table: table_data,
            choices,
        })
    }

    pub fn get_deletion_choice(&self) -> Option<DeletionChoices> {
        match self {
            PopupType::Choice(choice) => {
                DeletionChoices::from_repr(choice.table.state.selected().unwrap())
            }
            _ => None,
        }
    }
}
