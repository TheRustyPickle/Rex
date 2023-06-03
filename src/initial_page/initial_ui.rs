use crate::page_handler::{BACKGROUND, RED, TEXT};
use crate::utility::{create_bolded_text, main_block, styled_block};
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Modifier, Style};
use tui::widgets::Paragraph;
use tui::Frame;

/// The function draws the Initial page of the interface.
#[cfg(not(tarpaulin_include))]
pub fn initial_ui<B: Backend>(f: &mut Frame<B>, start_from: usize) {
    let size = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(8),
                Constraint::Length(1),
                Constraint::Min(5),
            ]
            .as_ref(),
        )
        .split(size);

    f.render_widget(main_block(), size);

    // This is the text that is shown in the startup which is the project's name in ASCII format.
    let text = r#"   _____    ______  __   __
  |  __ \  |  ____| \ \ / /
  | |__) | | |__     \ V / 
  |  _  /  |  __|     > <  
  | | \ \  | |____   / . \ 
  |_|  \_\ |______| /_/ \_\"#
        .to_string();

    // To work with this and add a slight touch of animation, we will split the entire
    // text by \n. Once it is done, we will loop through each line and take a specific amount of chars from each line.
    let splitted = text.split('\n').collect::<Vec<&str>>();
    let mut upper_text = String::new();

    for line in splitted {
        // if the line is 20 chars and the index is 15, take the chars from 15-20 and 0-3 indexes.
        // this var stores how many to take from 0 index
        let mut to_add_from_start = 0;
        // amount of chars per line
        let mut total_to_add = 10;

        if start_from + total_to_add > line.len() {
            let extra_index = (start_from + total_to_add) - line.len();
            // add extra index to take from beginning and remove from the starting point
            // if it will go out of bound
            total_to_add -= extra_index;
            to_add_from_start += extra_index;
        }

        // go through each char of the line
        for (index, char) in line.chars().enumerate() {
            if to_add_from_start != 0 {
                // add chars if we have to take anything from index 0 to something index
                upper_text.push(char);
                to_add_from_start -= 1;
            } else if total_to_add != 0 && index >= start_from {
                // if we are at the start point, take the char
                upper_text.push(char);
                total_to_add -= 1;
            } else if index != start_from || total_to_add == 0 {
                // if the 10 char limit is crossed, only add empty space
                upper_text.push(' ');
            }
        }
        upper_text.push('\n');
    }
    let unmodified_second_text = "Arrow Key: Navigate
F: Home Page
A: Add Transaction Page
T: Add Transfer Page
R: Chart Page
Z: Summary Page
D: Delete selected Transaction (Home Page)
J: Add/Rename/Reposition Transaction Methods (Home Page)
E: Edit Selected Transaction (Home Page)
H: Open Help (Use for detailed help)
Q: Quit

Add Transaction/Transfer Page:
1: Edit Date           
2: Edit TX details     
3: Edit TX/From Method
4: Edit Amount/To Method 
5: Edit TX Type/Amount
6: Edit Tags  
S: Save inputted data as a Transaction
C: Clear all fields
Enter: Submit a field and continue
Esc: Stop editing a filed

Other Pages:
Arrow Up/Down: Cycle widgets
Arrow Left/Right: Move value of the widget
";

    // bold a part of the text before rendering
    let second_text = create_bolded_text(unmodified_second_text);

    let middle_text = "Press Any Key To Continue";

    let paragraph = Paragraph::new(upper_text)
        .style(
            Style::default()
                .bg(BACKGROUND)
                .fg(TEXT)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);

    let paragraph_2 = Paragraph::new(middle_text)
        .style(
            Style::default()
                .bg(BACKGROUND)
                .fg(RED)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);

    let paragraph_3 = Paragraph::new(second_text)
        .style(Style::default().bg(BACKGROUND).fg(TEXT))
        .block(styled_block("Help"));

    f.render_widget(paragraph, chunks[0]);
    f.render_widget(paragraph_2, chunks[1]);
    f.render_widget(paragraph_3, chunks[2]);
}
