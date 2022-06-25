# Rex
A terminal based interface to keep track of income, expense and transactions writted in Rust and uses tui-rs. Stores the data in a local database and shows the changes that was made to the balance for each transaction on selection.

<img src="https://dl.dropboxusercontent.com/s/ecnixug3vus2bj7/Rex_1.png" alt="Rex_1" width="48%" > <img src="https://dl.dropboxusercontent.com/s/uzi0ft4aw5u68gf/Rex_2.png" alt="Rex_2" width="48%" >

<h4>Requirements</h4>
- Latest version of Rust

<h4>How to run</h4>
- Clone the repository
```
git clone https://github.com/WaffleMixer/Rex
```
- Run with Cargo
```
cargo run
```

<h4>Executables</h4>
To Be Released. Currently work in progress.

<h4>Hotkeys</h4>
May be subject to changes

'Q' : Quits the interface
'A' : Selects the Add Transaction interface
'H' : Selects the main Home page
'D' : If a transaction is selected on the Home page, deletes it
'S' : Works in the Transaction Interface to save a Transaction data
'Enter' : Submits the field data in Add Transaction interface
'Esc' : Submits the field data in Add Transaction interface

<h4>Currently in the works</h4>
- Initial UI
- Tighter transaction data checking
- Proper error handling
- Better Hotkey guide and Help
- Color scheme changes?
- Custom Transaction Methods. Currently only supports default values source_1 to 4
