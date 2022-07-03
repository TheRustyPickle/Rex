use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Alignment},
    style::{Color, Modifier, Style},
    text::{Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// The initial UI that starts on the startup of the program. The function
/// draws 2 widgets with the intention to show the hotkeys of the program.
pub fn starter_ui<B: Backend>(
    f: &mut Frame<B>,
    index: usize,
) {
    let size = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(8),
                Constraint::Min(5),
            ]
            .as_ref(),
        )
        .split(size);

    let block = Block::default().style(Style::default().bg(Color::White).fg(Color::Green));
    let create_block = |title| {
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::White).fg(Color::Green))
            .title(Span::styled(
                title,
                Style::default().add_modifier(Modifier::BOLD),
            ))
    };
    f.render_widget(block, size);

    // This is the text that is shown in the startup which is the project's name in ASCII format.
    let text = r#"  ____    _____  __  __
    |  _ \  | ____| \ \/ /
    | |_) | |  _|    \  / 
    |  _ <  | |___   /  \ 
    |_| \_\ |_____| /_/\_\
                          "#.to_string();

    // To work with this and add a slight touch of animation, we will split the entire
    // text by \n. Once it is done, we will loop through each line and add the chars in a string for rendering.
    let state = text.split("\n");
    let splitted = state.collect::<Vec<&str>>();
    let mut new_text = String::new();

    for line in splitted {
        // Let's take a look what each variable is does:
        // total_initial_to_add : Once the index goes to the end, we want to already start rendering
        // from the beginning so this variable contains the amount of chars from the beginning of text to add for rendering.
        let mut total_initial_to_add = 0;

        // total_to_add : This is the amount of chars to added for rendering each loop
        let mut total_to_add = 10;

        // This is where the chars selection from the start begins once the index goes above the text's
        // length. -1 is added because we are working with index which starts at 0.
        if index + total_to_add > line.len() {
            total_initial_to_add = index + total_to_add - 1 - line.len();

            // unsure why this part works but it makes the rendering a bit smoother.
            if total_initial_to_add > total_to_add-1 {
                total_initial_to_add = total_to_add-1
            }
        }

        // after each loop we keep track of the loop index and the text index we want to add for rendering.
        let mut cu_index = 0;
        let mut target_index = index;
        for char in line.chars() {
            if cu_index == target_index && total_to_add != 0 && target_index < line.len() {
                new_text.push(char);
                total_to_add -= 1;
                cu_index += 1;
                target_index += 1
            }
            else if total_initial_to_add != 0 {
                new_text.push(char);
                cu_index += 1;
                total_initial_to_add -= 1;
            }
            else {
                new_text.push_str(" ");
                cu_index += 1;
            }
        }
        new_text.push_str("\n");
    }

    new_text.push_str("\n    Press Any Key To Continue");
    // TODO add htokey changes here
    let second_text = "'Arrow Key' : Navigate
'A' : Add Transaction Page
'H' : Home Page
'D' : Delete Selected Transaction (Home Page)
'S' : Save the inputted data as a Transaction (Add Transaction Page)
'Q' : Quit

Add Transaction Page:
'1': Edit Date          '4': Edit Amount
'2': Edit TX details    '3': Edit TX Method
'5': Edit TX Type
'Enter' or 'Esc': Submit/Stop Editing Field
";

    let paragraph = Paragraph::new(new_text)
        .style(Style::default().bg(Color::White).fg(Color::Green))
        .alignment(Alignment::Center);

    let paragraph_2 = Paragraph::new(second_text)
    .style(Style::default().bg(Color::White).fg(Color::Green))
    .block(create_block("Help"));

    f.render_widget(paragraph, chunks[0]);
    f.render_widget(paragraph_2, chunks[1]);
    
}