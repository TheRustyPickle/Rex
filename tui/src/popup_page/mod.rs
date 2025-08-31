#[cfg(not(tarpaulin_include))]
mod popup_data;
#[cfg(not(tarpaulin_include))]
mod popup_ui;

pub use popup_data::{A, F, H, PopupData, Q, R, V, W, Y, Z};
pub use popup_ui::{create_deletion_popup, create_popup};
