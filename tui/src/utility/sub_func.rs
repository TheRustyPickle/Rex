use std::collections::HashMap;
use std::process::Command;

/// Tries to open terminal/cmd and run this app
/// Currently supports windows cmd, konsole, gnome-terminal, kgx (also known as gnome-console)
pub fn start_terminal(original_dir: &str) -> bool {
    let mut all_terminals = HashMap::new();

    if cfg!(target_os = "windows") {
        all_terminals.insert("cmd.exe", vec!["start".to_string(), "rex".to_string()]);
    } else {
        let gnome_dir = format!("--working-directory={original_dir}");

        all_terminals.insert(
            "konsole",
            vec![
                "--new-tab".to_string(),
                "--workdir".to_string(),
                original_dir.to_string(),
                "-e".to_string(),
                "./rex".to_string(),
            ],
        );

        all_terminals.insert(
            "gnome-terminal",
            vec![
                gnome_dir.clone(),
                "--maximize".to_string(),
                "--".to_string(),
                "./rex".to_string(),
            ],
        );

        all_terminals.insert(
            "kgx",
            vec![gnome_dir, "-e".to_string(), "./rex".to_string()],
        );
    }

    let mut terminal_opened = false;

    for (key, value) in all_terminals {
        let status = Command::new(key).args(value).output();
        if let Ok(out) = status
            && !out.stderr.len() > 2
        {
            terminal_opened = true;
            break;
        }
    }

    terminal_opened
}
