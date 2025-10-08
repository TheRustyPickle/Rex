use anyhow::{Result, anyhow};
use app::conn::DbConn;
use ratatui::Frame;
use ratatui::style::Color;
use rfd::FileDialog;
use std::path::PathBuf;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, FromRepr};

use crate::config::Config;
use crate::page_handler::{BLUE, RED, TableData};

pub enum PopupType {
    Info(InfoPopup),
    Choice(ChoicePopup),
    Reposition(RepositionPopup),
    Input,
    InputReposition,
    NewPaths(NewPathsPopup),
    Nothing,
}

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
    RepositionHelp,
    Error(String),
    ShowDetails(String),
}

pub struct InfoPopup {
    pub scroll_position: usize,
    pub max_scroll: usize,
    pub showing: InfoPopupState,
}

pub struct ChoicePopup {
    pub table: TableData,
    pub choices: Vec<ChoiceDetails>,
    pub showing: ChoicePopupState,
}

pub struct RepositionPopup {
    pub reposition_table: TableData,
    pub reposition_contents: Vec<String>,
    pub confirm_table: TableData,
    pub reposition_selected: bool,
}

pub struct NewPathsPopup {
    pub new_location: bool,
    pub paths: Vec<PathBuf>,
    pub table: TableData,
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

#[derive(Copy, Clone)]
pub enum ChoicePopupState {
    Delete,
    Config,
}

#[derive(EnumIter, Display, FromRepr, Copy, Clone)]
pub enum NewPathChoices {
    #[strum(to_string = "Select new path")]
    SelectNewPath,
    #[strum(to_string = "Clear path(s)")]
    ClearAll,
    #[strum(to_string = "Confirm")]
    Confirm,
}

#[derive(EnumIter, Display, FromRepr, Copy, Clone)]
pub enum ConfigChoices {
    #[strum(to_string = "Add new Transaction Method")]
    AddNewTxMethod,
    #[strum(to_string = "Rename a Transaction Method")]
    RenameTxMethod,
    #[strum(to_string = "Reposition Transaction Methods")]
    RepositionTxMethod,
    #[strum(to_string = "Set a new location for app data")]
    NewLocation,
    #[strum(to_string = "Set backup paths for app data")]
    BackupPaths,
}

impl InfoPopup {
    pub fn is_new_update(&self) -> bool {
        let new_update = matches!(self.showing, InfoPopupState::NewUpdate(_));
        new_update
    }
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

impl ConfigChoices {
    fn to_choice(self) -> ChoiceDetails {
        ChoiceDetails {
            text: self.to_string(),
            color: BLUE,
        }
    }
}

impl PopupType {
    pub fn show_ui(&mut self, f: &mut Frame) {
        match self {
            PopupType::Info(info) => info.show_ui(f),
            PopupType::Choice(choice) => choice.show_ui(f),
            PopupType::Reposition(reposition) => reposition.show_ui(f),
            PopupType::NewPaths(new_paths) => new_paths.show_ui(f),
            PopupType::Input | PopupType::InputReposition => todo!(),
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
            PopupType::NewPaths(new_paths) => {
                new_paths.table.next();
            }
            PopupType::Input | PopupType::InputReposition => todo!(),
            PopupType::Reposition(reposition) => {
                if reposition.reposition_selected {
                    let max_index = reposition.reposition_contents.len() - 1;

                    if reposition.reposition_table.state.selected().unwrap() == max_index {
                        reposition.reposition_selected = false;
                        reposition.confirm_table.state.select(Some(0));
                    } else {
                        reposition.reposition_table.next();
                    }
                } else {
                    reposition.confirm_table.state.select(None);
                    reposition.reposition_table.state.select(Some(0));
                    reposition.reposition_selected = true;
                }
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
            PopupType::NewPaths(new_paths) => {
                new_paths.table.previous();
            }
            PopupType::Input | PopupType::InputReposition => todo!(),
            PopupType::Reposition(reposition) => {
                if reposition.reposition_selected {
                    if reposition.reposition_table.state.selected().unwrap() == 0 {
                        reposition.reposition_selected = false;
                        reposition.confirm_table.state.select(Some(0));
                    } else {
                        reposition.reposition_table.previous();
                    }
                } else {
                    let max_index = reposition.reposition_contents.len() - 1;

                    reposition.confirm_table.state.select(None);
                    reposition.reposition_table.state.select(Some(max_index));
                    reposition.reposition_selected = true;
                }
            }
            PopupType::Nothing => {}
        }
    }

    pub fn move_down(&mut self) {
        match self {
            PopupType::Reposition(reposition) => {
                if !reposition.reposition_selected {
                    return;
                }

                let max_index = reposition.reposition_contents.len() - 1;

                let selected_index = reposition.reposition_table.state.selected().unwrap();

                if selected_index == max_index {
                    return;
                }

                reposition
                    .reposition_contents
                    .swap(selected_index, selected_index + 1);

                let table_items = reposition
                    .reposition_contents
                    .iter()
                    .map(|c| vec![c.clone()])
                    .collect();
                let table_data = TableData::new(table_items);

                reposition.reposition_table = table_data;

                reposition
                    .reposition_table
                    .state
                    .select(Some(selected_index + 1));
            }
            PopupType::Info(_) | PopupType::Choice(_) => {}
            _ => {}
        }
    }

    pub fn move_up(&mut self) {
        match self {
            PopupType::Reposition(reposition) => {
                if !reposition.reposition_selected {
                    return;
                }

                let selected_index = reposition.reposition_table.state.selected().unwrap();

                if selected_index == 0 {
                    return;
                }

                reposition
                    .reposition_contents
                    .swap(selected_index, selected_index - 1);

                let table_items = reposition
                    .reposition_contents
                    .iter()
                    .map(|c| vec![c.clone()])
                    .collect();
                let table_data = TableData::new(table_items);

                reposition.reposition_table = table_data;

                reposition
                    .reposition_table
                    .state
                    .select(Some(selected_index - 1));
            }
            PopupType::Info(_) | PopupType::Choice(_) => {}
            _ => {}
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
            showing: ChoicePopupState::Delete,
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

    pub fn new_choice_config() -> Self {
        let choices: Vec<ChoiceDetails> = ConfigChoices::iter()
            .map(ConfigChoices::to_choice)
            .collect();

        let table_items = choices.iter().map(|c| vec![c.text.clone()]).collect();
        let mut table_data = TableData::new(table_items);
        table_data.state.select(Some(0));

        PopupType::Choice(ChoicePopup {
            table: table_data,
            choices,
            showing: ChoicePopupState::Config,
        })
    }

    pub fn get_config_choice(&self) -> Option<ConfigChoices> {
        match self {
            PopupType::Choice(choice) => {
                ConfigChoices::from_repr(choice.table.state.selected().unwrap())
            }
            _ => None,
        }
    }

    pub fn new_reposition(conn: &mut DbConn) -> Result<Self> {
        let tx_methods = conn.get_tx_methods_sorted();

        if tx_methods.len() == 1 {
            return Err(anyhow!(
                "Needs at least 2 transaction methods to exist for repositioning"
            ));
        }

        let table_contents: Vec<String> = tx_methods.iter().map(|m| m.name.clone()).collect();

        let table_items = table_contents.iter().map(|c| vec![c.clone()]).collect();
        let mut table_data = TableData::new(table_items);
        table_data.state.select(Some(0));

        let confirmation_content = vec![vec!["Confirm".to_string()]];
        let confirmation_table_data = TableData::new(confirmation_content);

        let reposition_popup = RepositionPopup {
            reposition_table: table_data,
            reposition_contents: table_contents,
            confirm_table: confirmation_table_data,
            reposition_selected: true,
        };

        Ok(PopupType::Reposition(reposition_popup))
    }

    pub fn new_path(new_location: bool, config: &Config) -> Self {
        let choices: Vec<Vec<String>> = NewPathChoices::iter()
            .map(|c| vec![c.to_string()])
            .collect();

        let mut table_data = TableData::new(choices);
        table_data.state.select(Some(0));

        let paths = if new_location {
            if let Some(path) = &config.new_location {
                vec![path.clone()]
            } else {
                vec![]
            }
        } else if let Some(paths) = &config.backup_db_path {
            paths.clone()
        } else {
            vec![]
        };

        let popup = NewPathsPopup {
            new_location,
            table: table_data,
            paths,
        };

        Self::NewPaths(popup)
    }

    pub fn get_new_path_choice(&self) -> Option<NewPathChoices> {
        match self {
            PopupType::NewPaths(new_paths) => {
                NewPathChoices::from_repr(new_paths.table.state.selected().unwrap())
            }
            _ => None,
        }
    }

    pub fn add_new_path(&mut self, config: &Config) {
        let PopupType::NewPaths(new_paths) = self else {
            return;
        };

        let new_path = FileDialog::new().set_directory("~/").pick_folder();
        let Some(path) = new_path else {
            return;
        };

        let mut default_path = config.location.clone();
        default_path.pop();

        if path == default_path {
            return;
        }

        if new_paths.new_location {
            new_paths.paths = vec![path];
        } else if !new_paths.paths.contains(&path) {
            new_paths.paths.push(path);
        }
    }

    pub fn clear_paths(&mut self) {
        let PopupType::NewPaths(new_paths) = self else {
            return;
        };
        new_paths.paths.clear();
    }

    pub fn confirm_paths(&self, config: &mut Config) -> Result<()> {
        let PopupType::NewPaths(new_paths) = self else {
            return Err(anyhow!(
                "Should not have been called for this kind of popup"
            ));
        };

        if new_paths.new_location {
            config.set_new_location(new_paths.paths[0].clone())
        } else {
            config.set_backup_db_path(new_paths.paths.clone())
        }
    }
}
