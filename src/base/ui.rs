use std::{cell::RefCell, rc::Rc, io::{stdout}};
use eyre::Result;
use tokio::sync::mpsc::UnboundedReceiver;
use tui::{backend::{CrosstermBackend, Backend}, Terminal, Frame, layout::{Constraint, Direction, Layout, Alignment, Rect}, widgets::{Paragraph, Block, Borders, BorderType, Table, Row, Cell}, style::{Style, Color}, text::{Spans, Span}};

#[derive(Debug)]
pub struct App{
    pub cycle: usize,
    pub objf: f64
}
impl App{
    pub fn new()->Self{
        App{
            cycle: 0,
            objf: f64::INFINITY
        }
    }
    
}
pub fn start_ui(mut rx: UnboundedReceiver<App>) -> Result<()>{
    let stdout = stdout();
    crossterm::terminal::enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::new();

    terminal.clear()?;

    loop {
        
        app = match rx.try_recv(){
            Ok(s_app) => {
                s_app
            },
            Err(_) => app
        };

        terminal.draw(|rect| draw(rect, &app)).unwrap();

    }
    terminal.clear()?;
    terminal.show_cursor()?;
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}

pub fn draw<B>(rect: &mut Frame<B>, app: &App)
where
    B: Backend,
{
    let size = rect.size();
    check_size(&size);

    // Vertical layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(10)].as_ref())
        .split(size);

    // Title
    let title = draw_title();
    rect.render_widget(title, chunks[0]);

    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(20), Constraint::Length(32)].as_ref())
        .split(chunks[1]);

    let body = draw_body(false, &app);
    rect.render_widget(body, body_chunks[0]);

    // let help = draw_help(&app);
    // rect.render_widget(help, body_chunks[1]);
}

fn draw_title<'a>() -> Paragraph<'a> {
    Paragraph::new("NPcore Execution")
        .style(Style::default().fg(Color::LightCyan))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .border_type(BorderType::Plain),
        )
}
fn draw_body<'a>(loading: bool, state: &App) -> Paragraph<'a> {
    let loading_text = if loading { "Loading..." } else { "" };
    let cycle_text = format!("Cycle: {}", state.cycle);
    let objf_text = format!("-2LL: {}", state.objf);
    Paragraph::new(vec![
        Spans::from(Span::raw(loading_text)),
        Spans::from(Span::raw(cycle_text)),
        Spans::from(Span::raw(objf_text)),
    ])
    .style(Style::default().fg(Color::LightCyan))
    .alignment(Alignment::Left)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .border_type(BorderType::Plain),
    )
}

// fn draw_help(actions: &App) -> Table {
//     let key_style = Style::default().fg(Color::LightCyan);
//     let help_style = Style::default().fg(Color::Gray);

//     let mut rows = vec![];
//     for action in actions.actions().iter() {
//         let mut first = true;
//         for key in action.keys() {
//             let help = if first {
//                 first = false;
//                 action.to_string()
//             } else {
//                 String::from("")
//             };
//             let row = Row::new(vec![
//                 Cell::from(Span::styled(key.to_string(), key_style)),
//                 Cell::from(Span::styled(help, help_style)),
//             ]);
//             rows.push(row);
//         }
//     }

//     Table::new(rows)
//         .block(
//             Block::default()
//                 .borders(Borders::ALL)
//                 .border_type(BorderType::Plain)
//                 .title("Help"),
//         )
//         .widths(&[Constraint::Length(11), Constraint::Min(20)])
//         .column_spacing(1)
// }

fn check_size(rect: &Rect) {
    if rect.width < 52 {
        panic!("Require width >= 52, (got {})", rect.width);
    }
    if rect.height < 12 {
        panic!("Require height >= 12, (got {})", rect.height);
    }
}