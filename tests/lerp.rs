extern crate rex_tui;
use rex_tui::utility::LerpState;
use std::thread::sleep;
use std::time::Duration;

#[test]
fn lerp_test() {
    let mut lerp_state = LerpState::new(1.0);

    lerp_state.lerp("test1", 10.0);

    sleep(Duration::from_millis(100));

    let new_value = lerp_state.lerp("test1", 10.0);

    assert!(new_value < 10.0);
    assert!(lerp_state.has_active_lerps());

    sleep(Duration::from_millis(1000));

    let new_value = lerp_state.lerp("test1", 10.0);

    assert!(new_value == 10.0);
    assert!(!lerp_state.has_active_lerps());

    lerp_state.lerp("test1", 100.0);
    sleep(Duration::from_millis(100));

    lerp_state.clear();

    assert!(!lerp_state.has_active_lerps());
}
