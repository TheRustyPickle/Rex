use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Paragraph, Wrap};

use crate::pages::{A, F, H, J, Q, R, V, W, Y, Z};
use crate::theme::Theme;
use crate::utility::{create_bolded_text, main_block, styled_block};

/// The function draws the Initial page of the interface.
pub fn initial_ui(f: &mut Frame, start_from: usize, theme: &Theme) {
    let size = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(8),
            Constraint::Length(1),
            Constraint::Min(5),
        ])
        .split(size);

    let horizontal_help_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[2]);

    let help_chunk_1 = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(horizontal_help_chunks[0]);

    let help_chunk_2 = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(horizontal_help_chunks[1]);

    f.render_widget(main_block(theme), size);

    // This is the text that is shown in the startup which is the project's name in ASCII format.
    let text = r"   _____    ______  __   __
  |  __ \  |  ____| \ \ / /
  | |__) | | |__     \ V / 
  |  _  /  |  __|     > <  
  | | \ \  | |____   / . \ 
  |_|  \_\ |______| /_/ \_\"
        .to_string();

    // To work with this and add a slight touch of animation, we will split the entire
    // text by \n. Once it is done, we will loop through each line and take a specific amount of chars from each line.
    let split_text = text.split('\n').collect::<Vec<&str>>();
    let mut upper_text = String::new();

    for line in split_text {
        // If the line is 20 chars and the index is 15, take the chars from 15-20 and 0-3 indexes.
        // This var stores how many to take from 0 index
        let mut to_add_from_start = 0;
        // amount of chars per line
        let mut total_to_add = 10;

        if start_from + total_to_add > line.len() {
            let extra_index = (start_from + total_to_add) - line.len();
            // Add extra index to take from beginning and remove from the starting point
            // if it will go out of bound
            total_to_add -= extra_index;
            to_add_from_start += extra_index;
        }

        // Go through each char of the line
        for (index, char) in line.chars().enumerate() {
            if to_add_from_start != 0 {
                // Add chars if we have to take anything from index 0 to something index
                upper_text.push(char);
                to_add_from_start -= 1;
            } else if total_to_add != 0 && index >= start_from {
                // If we are at the start point, take the char
                upper_text.push(char);
                total_to_add -= 1;
            } else if index != start_from || total_to_add == 0 {
                // If the 10 char limit is crossed, only add empty space
                upper_text.push(' ');
            }
        }
        upper_text.push('\n');
    }

    let unmodified_first_help = format!(
        "{F}
{A}
{R}
{Z}
{Y}
{W}
{J}
{Q}
{H}"
    );

    let unmodified_second_help = "Arrow Up/Down: Cycle between widgets
Arrow Left/Right: Cycle values of a widget
H: Show help of the page the UI is currently on
X: Change sort type on summary/Change date type on Search page
Double R: Go to the Chart page and hide top widget. Press again to unhide
Double Z: Go to the Summary page and hide top widget. Press again to unhide";

    let unmodified_third_help = format!(
        "Arrow Up/Down: Move between year/month/transaction selections and scroll
Arrow Left/Right: Change values of the year/month selection
D: Delete selected Transaction
J: Take user input for various actions
E: Edit Selected Transaction
{V}
,: Move the transaction upward (Only if on the same date)
.: Move the transaction downward (Only if on the same date)"
    );

    let unmodified_fourth_help =
        "Arrow Left/Right: Move cursor left/right (If a field is selected)    
Arrow Up/Down: Go to the next value of the field (If a field is selected)
1: Edit Date  
2: Edit TX details
3: Edit TX Type
4: Edit TX Method
5: Edit Amount
6: Edit Tags
S: Save inputted data as a Transaction
C: Clear all fields and reset
X: Change date type (Only on search page)
Enter: Select the first field if nothing is selected
Enter: Verify, submit a field and continue
Esc: Stop editing a field";

    // Bold a part of the text before rendering
    let first_text = create_bolded_text(&unmodified_first_help);
    let second_text = create_bolded_text(unmodified_second_help);
    let third_text = create_bolded_text(&unmodified_third_help);
    let fourth_text = create_bolded_text(unmodified_fourth_help);

    let middle_text = "Press Any Key To Continue";

    let paragraph = Paragraph::new(upper_text)
        .style(
            Style::default()
                .bg(theme.background())
                .fg(theme.text())
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);

    let paragraph_2 = Paragraph::new(middle_text)
        .style(
            Style::default()
                .bg(theme.background())
                .fg(theme.negative())
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);

    let help_1 = Paragraph::new(first_text)
        .style(Style::default().bg(theme.background()).fg(theme.text()))
        .block(styled_block("Page Keys", theme))
        .wrap(Wrap { trim: true });

    let help_2 = Paragraph::new(second_text)
        .style(Style::default().bg(theme.background()).fg(theme.text()))
        .block(styled_block("Other Keys", theme))
        .wrap(Wrap { trim: true });

    let help_3 = Paragraph::new(third_text)
        .style(Style::default().bg(theme.background()).fg(theme.text()))
        .block(styled_block("Home Page Keys", theme))
        .wrap(Wrap { trim: true });

    let help_4 = Paragraph::new(fourth_text)
        .style(Style::default().bg(theme.background()).fg(theme.text()))
        .block(styled_block("Transaction Field Keys", theme))
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, chunks[0]);
    f.render_widget(paragraph_2, chunks[1]);
    f.render_widget(help_1, help_chunk_1[0]);
    f.render_widget(help_3, help_chunk_1[1]);
    f.render_widget(help_2, help_chunk_2[0]);
    f.render_widget(help_4, help_chunk_2[1]);
}
