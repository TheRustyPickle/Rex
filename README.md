<div align="center"><h1>Rex</h1></div>
<p align=center><a href="https://wakatime.com/badge/github/WaffleMixer/Rex"><img src="https://wakatime.com/badge/github/WaffleMixer/Rex.svg" alt="wakatime"></a></p>

A terminal based interface to keep track of income, expense and transactions, written in Rust and uses tui-rs for the interface. Stores the data in a local sqlite database and works fully offline.

<img src="https://dl.dropboxusercontent.com/s/ecnixug3vus2bj7/Rex_1.png" alt="Rex_1" width="48%" > <img src="https://dl.dropboxusercontent.com/s/uzi0ft4aw5u68gf/Rex_2.png" alt="Rex_2" width="48%" >



<h4>How to run Option 1</h4>

- Clone the repository
`
git clone https://github.com/WaffleMixer/Rex
`
- Run with Cargo
`
cargo run
`

<h4>How to run Option 2</h4>

- Download the latest executable from [Releases](https://github.com/WaffleMixer/Rex/releases).
  - Open terminal/CMD and start the program by the command `./rex` or `rex` depending on the OS at the correct directory.
  
  or
  
  - Double click the executable which will try to open a terminal/CMD by itself. 

<h4>How to run Option 3</h4>

- Run `cargo install --git https://github.com/WaffleMixer/Rex.git`
- Make sure `~/.cargo/bin` is on path if you're on linux

<h4>App Data Location</h4>

[See Here](https://docs.rs/dirs/latest/dirs/fn.data_local_dir.html) for location info which is determined based on the OS. If you are using an old version, you can manually move `data.sqlite` to the directory above to use it with existing data.

<h4>Status</h4>

More features are unlikely to be added unless something useful is suggested. 
