<p align="center">
  <img src="logo.png" width=400>
</p>

<div align="center"><h1>Rex</h1></div>
<div align="center">
<a href="https://wakatime.com/@RustyPickle"><img src="https://wakatime.com/badge/github/TheRustyPickle/Rex.svg" alt="wakatime"></a>
<a href="https://crates.io/crates/rex-tui">
<img src="https://img.shields.io/crates/v/rex-tui.svg?style=flat-square&logo=rust&color=orange"/></a>
<a href="https://github.com/TheRustyPickle/Rex/releases">
<img src="https://img.shields.io/github/v/release/TheRustyPickle/Rex?style=flat-square&logo=github&color=orange"/></a>
<a href="https://crates.io/crates/rex-tui">
<img src="https://img.shields.io/crates/d/rex-tui?style=flat-square"/></a>
</div>
Rex is a terminal user interface app for managing incomes, expenses, and transactions. Built with Rust and Ratatui with a simple interface that's easy to use.

![Rex](https://github.com/TheRustyPickle/Rex/assets/35862475/78fa9d86-5f7c-4c37-be35-437ecc8c4f92)

<h2>Key Features</h2>

* Easily view, add, edit, and delete transactions
* Navigate through transactions and instantly observe balance changes after each transaction
* Chart for visualizing balance changes over specific a month, year, or all transactions
* Access a summary with key insights with various info on income, expense, and percentage distribution.

* Built using SQLite database and keeps everything local
* Find transactions quickly using partial or specific information
* Organize transactions with custom tags for easy filtering
* Works fully offline

<h2>Installtion</h2>

**1. Run from Source Code:**

* Clone the repository
`
git clone https://github.com/TheRustyPickle/Rex
`
* Run with Cargo
`
cargo run --release
`

**2. Run the Latest Release:**

* Download the latest executable from [Releases](https://github.com/TheRustyPickle/Rex/releases).
  * Open terminal/CMD and start the program by the command `./rex` or `rex` depending on the OS at the correct directory.
  
  or
  
  * Double click the executable which will try to open a terminal/CMD by itself.

**3. Install from Cargo:**

* Run `cargo install rex-tui`
* Make sure `~/.cargo/bin` is on the PATH if you're on Linux

**4. Install using a package manager:**

* On NetBSD a package is available from the [official repositories](https://pkgsrc.se/finance/rex). To install it simply run:
```sh
pkgin install rex
```

<h2>App Data Location</h2>

See [here](https://docs.rs/dirs/latest/dirs/fn.data_local_dir.html) for location info where Rex data is saved which is determined based on the OS.

<h2>Feedback & Bug Reports</h2>

For any feedback, improvement suggestions, or bugs please [open an issue](https://github.com/TheRustyPickle/Rex/issues/new)
