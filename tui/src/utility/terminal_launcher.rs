use std::env;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};

const LAUNCHED_ENV: &str = "REX_LAUNCHED_IN_TERMINAL";

const STARTUP_GRACE: Duration = Duration::from_millis(400);

#[derive(Clone, Copy, PartialEq, Debug)]
enum Platform {
    Windows,
    MacOs,
    Unix,
}

impl Platform {
    fn current() -> Self {
        if cfg!(target_os = "windows") {
            Platform::Windows
        } else if cfg!(target_os = "macos") {
            Platform::MacOs
        } else {
            Platform::Unix
        }
    }
}

#[derive(Debug, PartialEq)]
struct Launcher {
    program: OsString,
    args: Vec<OsString>,
}

impl Launcher {
    fn new(program: &str, args: Vec<OsString>) -> Self {
        Self {
            program: program.into(),
            args,
        }
    }
}

/// Whether this process was started by `start_terminal` in an earlier run
#[must_use]
pub fn already_relaunched() -> bool {
    env::var_os(LAUNCHED_ENV).is_some()
}

/// Tries to open a terminal running this app, using the first one that works
#[must_use]
pub fn start_terminal(work_dir: &Path) -> bool {
    let exe = env::current_exe().unwrap_or_else(|_| PathBuf::from("rex"));
    let terminal_env = env::var("TERMINAL").ok();
    let desktop = env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();

    candidates(
        Platform::current(),
        &exe,
        work_dir,
        terminal_env.as_deref(),
        &desktop,
    )
    .iter()
    .any(|launcher| try_launch(launcher, work_dir))
}

fn candidates(
    platform: Platform,
    exe: &Path,
    work_dir: &Path,
    terminal_env: Option<&str>,
    desktop: &str,
) -> Vec<Launcher> {
    match platform {
        Platform::Windows => vec![Launcher::new(
            "cmd.exe",
            args(["/C", "start", ""], [exe.as_os_str()]),
        )],
        Platform::MacOs => vec![Launcher::new(
            "open",
            args(["-a", "Terminal"], [exe.as_os_str()]),
        )],
        Platform::Unix => unix_candidates(exe, work_dir, terminal_env, desktop),
    }
}

fn unix_candidates(
    exe: &Path,
    work_dir: &Path,
    terminal_env: Option<&str>,
    desktop: &str,
) -> Vec<Launcher> {
    let exe = exe.as_os_str();
    let dir = work_dir.as_os_str();

    let mut launchers = vec![
        Launcher::new("konsole", args(["--workdir"], [dir, OsStr::new("-e"), exe])),
        Launcher::new(
            "gnome-terminal",
            vec![joined("--working-directory=", dir), "--".into(), exe.into()],
        ),
        Launcher::new(
            "kgx",
            vec![joined("--working-directory=", dir), "-e".into(), exe.into()],
        ),
        Launcher::new(
            "xfce4-terminal",
            vec![joined("--working-directory=", dir), "-x".into(), exe.into()],
        ),
        Launcher::new(
            "mate-terminal",
            vec![joined("--working-directory=", dir), "-x".into(), exe.into()],
        ),
        Launcher::new(
            "lxterminal",
            vec![joined("--working-directory=", dir), "-e".into(), exe.into()],
        ),
        Launcher::new("tilix", args(["-w"], [dir, OsStr::new("-e"), exe])),
        Launcher::new(
            "terminator",
            vec![joined("--working-directory=", dir), "-x".into(), exe.into()],
        ),
        Launcher::new(
            "alacritty",
            args(["--working-directory"], [dir, OsStr::new("-e"), exe]),
        ),
        Launcher::new("kitty", args(["--directory"], [dir, exe])),
        Launcher::new(
            "wezterm",
            args(["start", "--cwd"], [dir, OsStr::new("--"), exe]),
        ),
        Launcher::new("foot", args(["-D"], [dir, exe])),
        Launcher::new("x-terminal-emulator", args(["-e"], [exe])),
        Launcher::new("xterm", args(["-e"], [exe])),
    ];

    let preferred = preferred_programs(terminal_env, desktop);

    prefer(&mut launchers, &preferred);

    if let Some(program) = terminal_program(terminal_env)
        && !launchers.iter().any(|l| l.program == *program)
    {
        let mut command = terminal_args(terminal_env);
        command.extend([OsString::from("-e"), exe.into()]);

        launchers.insert(
            0,
            Launcher {
                program: program.into(),
                args: command,
            },
        );
    }

    launchers
}

fn preferred_programs(terminal_env: Option<&str>, desktop: &str) -> Vec<String> {
    let mut preferred = Vec::new();

    if let Some(program) = terminal_program(terminal_env) {
        preferred.push(program.to_string_lossy().into_owned());
    }

    let desktop = desktop.to_lowercase();

    let native: &[&str] = if desktop.contains("kde") {
        &["konsole"]
    } else if ["gnome", "ubuntu", "unity", "budgie", "cinnamon"]
        .iter()
        .any(|d| desktop.contains(d))
    {
        &["gnome-terminal", "kgx"]
    } else if desktop.contains("xfce") {
        &["xfce4-terminal"]
    } else if desktop.contains("mate") {
        &["mate-terminal"]
    } else if desktop.contains("lxqt") || desktop.contains("lxde") {
        &["lxterminal"]
    } else {
        &[]
    };

    preferred.extend(native.iter().map(ToString::to_string));
    preferred
}

// Re-order `launchers` so that the preferred ones come first
fn prefer(launchers: &mut Vec<Launcher>, preferred: &[String]) {
    let mut front = Vec::new();

    for name in preferred {
        if let Some(index) = launchers.iter().position(|l| l.program == name.as_str()) {
            front.push(launchers.remove(index));
        }
    }

    front.append(launchers);
    *launchers = front;
}

fn terminal_program(terminal_env: Option<&str>) -> Option<&OsStr> {
    Path::new(terminal_env?.split_whitespace().next()?).file_name()
}

fn terminal_args(terminal_env: Option<&str>) -> Vec<OsString> {
    terminal_env
        .map(|value| {
            value
                .split_whitespace()
                .skip(1)
                .map(OsString::from)
                .collect()
        })
        .unwrap_or_default()
}

fn args<const A: usize, const B: usize>(first: [&str; A], second: [&OsStr; B]) -> Vec<OsString> {
    first
        .into_iter()
        .map(OsString::from)
        .chain(second.into_iter().map(OsString::from))
        .collect()
}

/// `prefix` immediately followed by `value`, like `--working-directory=/some/dir`
fn joined(prefix: &str, value: &OsStr) -> OsString {
    let mut joined = OsString::from(prefix);
    joined.push(value);
    joined
}

fn try_launch(launcher: &Launcher, work_dir: &Path) -> bool {
    let child = Command::new(&launcher.program)
        .args(&launcher.args)
        .current_dir(work_dir)
        .env(LAUNCHED_ENV, "1")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();

    match child {
        Ok(mut child) => started_successfully(&mut child, STARTUP_GRACE),
        // Not installed
        Err(_) => false,
    }
}

fn started_successfully(child: &mut Child, grace: Duration) -> bool {
    let started = Instant::now();

    loop {
        match child.try_wait() {
            Ok(Some(status)) => return status.success(),
            Ok(None) if started.elapsed() >= grace => return true,
            Ok(None) => sleep(Duration::from_millis(25)),
            Err(_) => return false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exe() -> PathBuf {
        PathBuf::from("/opt/rex/rex")
    }

    fn dir() -> PathBuf {
        PathBuf::from("/home/user/projects")
    }

    fn unix(terminal_env: Option<&str>, desktop: &str) -> Vec<Launcher> {
        candidates(Platform::Unix, &exe(), &dir(), terminal_env, desktop)
    }

    fn find<'a>(launchers: &'a [Launcher], program: &str) -> &'a Launcher {
        launchers.iter().find(|l| l.program == program).unwrap()
    }

    fn os(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn launchers_run_the_real_executable_not_a_relative_path() {
        for launcher in unix(None, "") {
            assert!(
                launcher.args.contains(&OsString::from("/opt/rex/rex")),
                "{:?} doesn't run the absolute executable path",
                launcher.program
            );
        }
    }

    #[test]
    fn known_terminals_get_their_own_flags() {
        let launchers = unix(None, "");

        assert_eq!(
            find(&launchers, "konsole").args,
            os(&["--workdir", "/home/user/projects", "-e", "/opt/rex/rex"])
        );
        assert_eq!(
            find(&launchers, "gnome-terminal").args,
            os(&[
                "--working-directory=/home/user/projects",
                "--",
                "/opt/rex/rex"
            ])
        );
        assert_eq!(
            find(&launchers, "xfce4-terminal").args,
            os(&[
                "--working-directory=/home/user/projects",
                "-x",
                "/opt/rex/rex"
            ])
        );
        assert_eq!(
            find(&launchers, "kitty").args,
            os(&["--directory", "/home/user/projects", "/opt/rex/rex"])
        );
        assert_eq!(
            find(&launchers, "wezterm").args,
            os(&[
                "start",
                "--cwd",
                "/home/user/projects",
                "--",
                "/opt/rex/rex"
            ])
        );
        assert_eq!(find(&launchers, "xterm").args, os(&["-e", "/opt/rex/rex"]));
    }

    #[test]
    fn default_order_is_stable_and_ends_with_xterm() {
        let first = unix(None, "");
        let second = unix(None, "");

        assert_eq!(first, second);
        assert_eq!(first.first().unwrap().program, "konsole");
        assert_eq!(first.last().unwrap().program, "xterm");
    }

    #[test]
    fn desktop_picks_its_own_terminal_first() {
        for (desktop, expected) in [
            ("KDE", "konsole"),
            ("GNOME", "gnome-terminal"),
            ("ubuntu:GNOME", "gnome-terminal"),
            ("X-Cinnamon", "gnome-terminal"),
            ("XFCE", "xfce4-terminal"),
            ("MATE", "mate-terminal"),
            ("LXQt", "lxterminal"),
        ] {
            assert_eq!(unix(None, desktop).first().unwrap().program, expected);
        }
    }

    #[test]
    fn gnome_falls_back_to_kgx_before_unrelated_terminals() {
        let programs: Vec<_> = unix(None, "GNOME")
            .into_iter()
            .map(|l| l.program)
            .take(2)
            .collect();

        assert_eq!(programs, vec!["gnome-terminal", "kgx"]);
    }

    #[test]
    fn terminal_env_with_a_known_name_uses_that_terminals_flags() {
        let launchers = unix(Some("/usr/bin/kitty"), "KDE");

        // kitty doesn't take -e, so it must use its own recipe rather than the generic one
        assert_eq!(launchers[0].program, "kitty");
        assert_eq!(
            launchers[0].args,
            os(&["--directory", "/home/user/projects", "/opt/rex/rex"])
        );
        // Nothing is duplicated
        assert_eq!(launchers.iter().filter(|l| l.program == "kitty").count(), 1);
    }

    #[test]
    fn unknown_terminal_env_is_tried_first_with_dash_e_and_its_own_args() {
        let launchers = unix(Some("mycoolterm --fast"), "KDE");

        assert_eq!(launchers[0].program, "mycoolterm");
        assert_eq!(launchers[0].args, os(&["--fast", "-e", "/opt/rex/rex"]));
        assert_eq!(launchers[1].program, "konsole");
    }

    #[test]
    fn blank_terminal_env_is_ignored() {
        assert_eq!(unix(Some(""), ""), unix(None, ""));
        assert_eq!(unix(Some("   "), ""), unix(None, ""));
    }

    #[test]
    fn macos_and_windows_have_a_single_launcher() {
        let mac = candidates(Platform::MacOs, &exe(), &dir(), None, "");
        assert_eq!(
            mac,
            vec![Launcher::new(
                "open",
                os(&["-a", "Terminal", "/opt/rex/rex"])
            )]
        );

        let windows = candidates(Platform::Windows, &exe(), &dir(), None, "");
        assert_eq!(
            windows,
            vec![Launcher::new(
                "cmd.exe",
                os(&["/C", "start", "", "/opt/rex/rex"])
            )]
        );
    }

    #[cfg(unix)]
    mod launching {
        use super::*;

        fn launcher(program: &str, args: &[&str]) -> Launcher {
            Launcher::new(program, os(args))
        }

        #[test]
        fn a_program_that_exits_cleanly_counts_as_started() {
            assert!(try_launch(&launcher("true", &[]), &std::env::temp_dir()));
        }

        #[test]
        fn a_program_that_exits_with_an_error_is_skipped() {
            assert!(!try_launch(&launcher("false", &[]), &std::env::temp_dir()));
        }

        #[test]
        fn a_missing_program_is_skipped() {
            assert!(!try_launch(
                &launcher("rex-definitely-not-a-terminal", &[]),
                &std::env::temp_dir()
            ));
        }

        #[test]
        fn a_program_still_running_after_the_grace_period_counts_as_started() {
            let mut child = Command::new("sleep").arg("5").spawn().unwrap();

            assert!(started_successfully(&mut child, Duration::from_millis(100)));

            child.kill().unwrap();
            child.wait().unwrap();
        }

        #[test]
        fn the_launched_environment_marker_is_passed_down() {
            let marker = std::env::temp_dir().join("rex_launcher_marker_test.txt");
            let _ = std::fs::remove_file(&marker);

            let script = format!("printf %s \"${LAUNCHED_ENV}\" > {}", marker.display());
            assert!(try_launch(
                &launcher("sh", &["-c", &script]),
                &std::env::temp_dir()
            ));

            assert_eq!(std::fs::read_to_string(&marker).unwrap(), "1");
            std::fs::remove_file(marker).unwrap();
        }
    }
}
