use eyre::Result;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::Span,
    widgets::{
        Axis, Block, BorderType, Borders, Cell, Chart, Dataset, GraphType, Paragraph, Row, Table,
    },
    Frame, Terminal,
};
use std::{
    io::stdout,
    time::{Duration, Instant},
};
use tokio::sync::mpsc::UnboundedReceiver;

use super::{
    inputs::{events::Events, InputEvent},
    state::AppHistory,
    state::AppState,
    App, AppReturn,
};

pub fn start_ui(mut rx: UnboundedReceiver<AppState>) -> Result<()> {
    let stdout = stdout();
    crossterm::terminal::enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::new();
    let mut app_history = AppHistory::new();

    terminal.clear()?;

    // User event handler
    let tick_rate = Duration::from_millis(200);
    let mut events = Events::new(tick_rate);

    let mut start_time = Instant::now();
    let mut elapsed_time = Duration::from_secs(0);

    loop {
        app.state = match rx.try_recv() {
            Ok(state) => state,
            Err(_) => app.state,
        };

        // Stop incrementing elapsed time if conv is true
        if app.state.stop_text.is_empty() {
            let now = Instant::now();
            if now.duration_since(start_time) > tick_rate {
                elapsed_time += now.duration_since(start_time);
                start_time = now;
            }
        }

        if !app_history
            .cycles
            .iter()
            .any(|state| state.cycle == app.state.cycle)
        {
            app_history.add_cycle(app.state.clone());
        }

        terminal
            .draw(|rect| draw(rect, &app, &app_history, elapsed_time))
            .unwrap();

        // Handle inputs
        let result = match events.recv() {
            Some(InputEvent::Input(key)) => app.do_action(key),
            None => AppReturn::Continue,
        };
        // Check if we should exit
        if result == AppReturn::Exit {
            break;
        }
    }

    terminal.clear()?;
    terminal.show_cursor()?;
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}

pub fn draw<B>(rect: &mut Frame<B>, app: &App, app_history: &AppHistory, elapsed_time: Duration)
where
    B: Backend,
{
    let size = rect.size();
    check_size(&size);

    // Vertical layout (overall)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Min(5),
            ]
            .as_ref(),
        )
        .split(size);

    // Title in first chunk (top)
    let title = draw_title();
    rect.render_widget(title, chunks[0]);

    // Horizontal layout for three chunks (middle)
    let body_chunk = chunks[1];
    let body_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(40),
                Constraint::Percentage(40),
                Constraint::Percentage(20),
            ]
            .as_ref(),
        )
        .split(body_chunk);

    // First chunk
    let status = draw_status(app, elapsed_time);
    rect.render_widget(status, body_layout[0]);

    // Second chunk
    let options = draw_options();
    rect.render_widget(options, body_layout[1]);

    // Third chunk
    let commands = draw_commands(app);
    rect.render_widget(commands, body_layout[2]);

    // Prepare the data
    let data: Vec<(f64, f64)> = app_history
        .cycles
        .iter()
        .enumerate()
        .map(|(x, entry)| (x as f64, entry.objf))
        .collect();

    let start_index = (data.len() as f64 * 0.1) as usize;

    // Calculate data points and remove infinities
    let mut norm_data: Vec<(f64, f64)> = data
        .iter()
        .filter(|&(_, y)| !y.is_infinite())
        .skip(start_index)
        .map(|&(x, y)| (x, y))
        .collect();

    let plot = draw_plot(&mut norm_data);
    rect.render_widget(plot, chunks[2]);
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

fn draw_status<'a>(app: &App, elapsed_time: Duration) -> Table<'a> {
    // Define (formatted) texts
    let cycle_text = format!("{}", app.state.cycle);
    let objf_text = format!("{:.5}", app.state.objf);
    let delta_objf_text = format!("{:.5}", app.state.delta_objf);
    let gamma_text = format!("{:.5}", app.state.gamlam);
    let spp_text = format!("{}", app.state.nspp);
    let time_text = format_time(elapsed_time);
    let stop_text = format!("{}", app.state.stop_text);

    // Define the table data
    let data = vec![
        ("Current cycle", cycle_text),
        ("Objective function", objf_text),
        ("Δ Objective function", delta_objf_text),
        ("Gamma/Lambda", gamma_text),
        ("Support points", spp_text),
        ("Elapsed time", time_text),
        ("Convergence", stop_text),
        // Add more rows as needed
    ];

    // Populate the table rows
    let rows: Vec<Row> = data
        .iter()
        .map(|(key, value)| {
            let title_style = Style::default().add_modifier(Modifier::BOLD);
            let title_cell = Cell::from(Span::styled(format!("{}:", key), title_style));
            let value_cell = Cell::from(value.to_string());
            Row::new(vec![title_cell, value_cell])
        })
        .collect();

    // Create the table widget
    Table::new(rows)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Plain)
                .title(" Status "),
        )
        .widths(&[Constraint::Percentage(50), Constraint::Percentage(50)]) // Set percentage widths for columns
        .column_spacing(1)
}

fn draw_options<'a>() -> Table<'a> {
    // Define the table data
    let data = vec![
        ("Maximum cycles", "Placeholder"),
        ("Engine", "NPAG"),
        ("Convergence criteria", "Placeholder"),
        ("Initial gridpoints", "Placeholder"),
        ("Error model", "Placeholder"),
        ("Cache", "Placeholder"),
        ("Random seed", "Placeholder"),
        // Add more rows as needed
    ];

    // Populate the table rows
    let rows: Vec<Row> = data
        .iter()
        .map(|(key, value)| {
            let title_style = Style::default().add_modifier(Modifier::BOLD);
            let title_cell = Cell::from(Span::styled(format!("{}:", key), title_style));
            let value_cell = Cell::from(value.to_string());
            Row::new(vec![title_cell, value_cell])
        })
        .collect();

    // Create the table widget
    Table::new(rows)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Plain)
                .title(" Options "),
        )
        .widths(&[Constraint::Percentage(50), Constraint::Percentage(50)]) // Set percentage widths for columns
        .column_spacing(1)
}

fn draw_commands(app: &App) -> Table {
    let key_style = Style::default().fg(Color::LightCyan);
    let help_style = Style::default().fg(Color::Gray);

    let mut rows = vec![];
    for action in app.actions.actions().iter() {
        let mut first = true;
        for key in action.keys() {
            let help = if first {
                first = false;
                action.to_string()
            } else {
                String::from("")
            };
            let row = Row::new(vec![
                Cell::from(Span::styled(key.to_string(), key_style)),
                Cell::from(Span::styled(help, help_style)),
            ]);
            rows.push(row);
        }
    }

    Table::new(rows)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Plain)
                .title(" Commands "),
        )
        .widths(&[Constraint::Percentage(50), Constraint::Percentage(50)]) // Set percentage widths for columns
        .column_spacing(1)
}

fn draw_plot(norm_data: &mut Vec<(f64, f64)>) -> Chart {
    // Find min and max values
    let (x_min, x_max) = norm_data
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), (x, _)| {
            (min.min(*x), max.max(*x))
        });

    let (y_min, y_max) = norm_data
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), (_, y)| {
            (min.min(*y), max.max(*y))
        });

    // Compute the dynamic step size for the X-labels
    let step_size = ((x_max - x_min) / 10.0).max(1.0).ceil();

    // Generate X-labels using the dynamic step size
    let x_labels: Vec<Span> = ((x_min as i64)..=(x_max as i64))
        .step_by(step_size as usize)
        .map(|x| Span::from(x.to_string()))
        .collect();

    // Generate four Y-labels, evenly from y_min to y_max
    let y_step = (y_max - y_min) / 5.0; // To get 4 labels, we need 3 steps
    let y_labels: Vec<Span> = (0..=3)
        .map(|i| {
            let y = y_min + y_step * (i as f64);
            Span::from(format!("{:.2}", y)) // format the y value to 2 decimal places
        })
        .collect();

    // Prepare the dataset
    let dataset = vec![Dataset::default()
        .name("-2LL")
        .marker(symbols::Marker::Dot)
        .style(Style::default().fg(Color::Cyan))
        .graph_type(GraphType::Scatter)
        .data(norm_data)];

    // Return the plot
    Chart::new(dataset)
        .x_axis(
            Axis::default()
                .title("Cycle")
                .bounds([x_min, x_max])
                .labels(x_labels),
        )
        .y_axis(
            Axis::default()
                .title("-2LL")
                .bounds([y_min, y_max])
                .labels(y_labels),
        )
        .block(
            Block::default()
                .title(" Objective function ")
                .borders(Borders::ALL),
        )
}

fn check_size(rect: &Rect) {
    if rect.width < 52 {
        // panic!("Require width >= 52, (got {})", rect.width);
    }
    if rect.height < 12 {
        // panic!("Require height >= 12, (got {})", rect.height);
    }
}

fn format_time(elapsed_time: std::time::Duration) -> String {
    let elapsed_seconds = elapsed_time.as_secs();
    let (elapsed, unit) = if elapsed_seconds < 60 {
        (elapsed_seconds, "s")
    } else if elapsed_seconds < 3600 {
        let elapsed_minutes = elapsed_seconds / 60;
        (elapsed_minutes, "m")
    } else {
        let elapsed_hours = elapsed_seconds / 3600;
        (elapsed_hours, "h")
    };
    let time_text = format!("{}{}", elapsed, unit);
    time_text
}
