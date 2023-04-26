<div align="center"><h1>Rex</h1></div>
<div align="center">
<a href="https://wakatime.com/@RustyPickle"><img src="https://wakatime.com/badge/github/TheRustyPickle/Rex.svg" alt="wakatime"></a>
<a href="https://crates.io/crates/rex-tui">
<img src="https://img.shields.io/crates/v/rex-tui.svg?style=flat-square&logo=rust&color=orange"/></a>
<a href="https://github.com/TheRustyPickle/Rex/releases">
<img src="https://img.shields.io/github/v/release/TheRustyPickle/Rex?style=flat-square&logo=github&color=orange"/></a>
<a href="https://crates.io/crates/rex-tui">
<img src="https://img.shields.io/crates/d/rex-tui.svg?style=flat-square"/></a>
</div>
Rex is a terminal user interface app for managing income, expenses, and transactions. Built with Rust and tui-rs with a simple interface that's easy to use.

![new_rex](https://user-images.githubusercontent.com/35862475/234666900-317aaece-6955-4e15-a92b-b4cb2d3daf4a.png)


<h2>Getting Started</h2>

**1. Run from Source Code:**
* Clone the repository
`
git clone https://github.com/TheRustyPickle/Rex
`
* Run with Cargo
`
cargo run -- release
`

**2. Run the Latest Release:**
* Download the latest executable from [Releases](https://github.com/TheRustyPickle/Rex/releases).
  * Open terminal/CMD and start the program by the command `./rex` or `rex` depending on the OS at the correct directory.
  
  or
  
  * Double click the executable which will try to open a terminal/CMD by itself. 

**3. Install from Cargo:**
* Run `cargo install --git https://github.com/TheRustyPickle/Rex.git`
* Make sure `~/.cargo/bin` is on path if you're on linux

<h2>App Data Location</h2>

See [here](https://docs.rs/dirs/latest/dirs/fn.data_local_dir.html) for location info where Rex data is saved which is determined based on the OS. 

<h2>Feedback & Bug Reports</h2>

For any feedback, improvement suggestions or bugs please [open an issue](https://github.com/TheRustyPickle/Rex/issues/new)
