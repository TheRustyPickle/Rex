#[cfg(not(tarpaulin_include))]
mod initial_ui;
#[cfg(not(tarpaulin_include))]
mod version_checker;

pub use initial_ui::initial_ui;
pub use version_checker::check_version;
