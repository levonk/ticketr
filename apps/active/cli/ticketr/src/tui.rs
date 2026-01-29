use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use tokio::sync::mpsc;
use crate::ticket::{Ticket, TicketManager};

#[derive(Clone, Debug)]
pub enum AppEvent {
    Refresh,
    Quit,
    SelectTicket(usize),
    ChangeStatus(String),
    ShowHelp,
}

#[derive(Clone, Debug)]
pub enum AppState {
    Normal,
    Help,
    CreatingTicket,
    EditingTicket(usize),
}

pub struct App {
    pub tickets: Vec<Ticket>,
    pub selected_ticket: usize,
    pub state: AppState,
    pub status_filter: Option<String>,
    list_state: ListState,
}

impl App {
    pub fn new() -> Self {
        Self {
            tickets: Vec::new(),
            selected_ticket: 0,
            state: AppState::Normal,
            status_filter: None,
            list_state: ListState::default(),
        }
    }

    pub fn next(&mut self) {
        if !self.tickets.is_empty() {
            self.selected_ticket = (self.selected_ticket + 1) % self.tickets.len();
        }
    }

    pub fn previous(&mut self) {
        if !self.tickets.is_empty() {
            self.selected_ticket = if self.selected_ticket == 0 {
                self.tickets.len() - 1
            } else {
                self.selected_ticket - 1
            };
        }
    }

    pub fn update_tickets(&mut self, tickets: Vec<Ticket>) {
        self.tickets = tickets;
        if self.selected_ticket >= self.tickets.len() && !self.tickets.is_empty() {
            self.selected_ticket = self.tickets.len() - 1;
        }
    }
}

pub async fn run_tui(manager: &mut TicketManager) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new();

    // Load initial tickets
    let tickets = manager.list_tickets()?;
    app.update_tickets(tickets);

    // Setup event handling
    let (tx, mut rx) = mpsc::channel::<AppEvent>(100);

    // Clone manager for async operations
    let manager_clone = manager.clone();
    let tx_clone = tx.clone();

    // Spawn background task for ticket updates
    tokio::spawn(async move {
        let mut manager = manager_clone;
        let mut last_update = std::time::Instant::now();
        
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            
            if let Ok(tickets) = manager.list_tickets() {
                let _ = tx_clone.send(AppEvent::Refresh);
            }
        }
    });

    // Main UI loop
    loop {
        // Draw UI
        terminal.draw(|f| ui(f, &app))?;

        // Handle events
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match app.state {
                    AppState::Normal => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Char('j') | KeyCode::Down => app.next(),
                        KeyCode::Char('k') | KeyCode::Up => app.previous(),
                        KeyCode::Char('h') | KeyCode::Left => {
                            app.state = AppState::Help;
                        }
                        KeyCode::Char('r') => {
                            if let Ok(tickets) = manager.list_tickets() {
                                app.update_tickets(tickets);
                            }
                        }
                        KeyCode::Char('1') => {
                            if let Some(ticket) = app.tickets.get(app.selected_ticket) {
                                let _ = manager.update_status(&ticket.id, "open");
                                if let Ok(tickets) = manager.list_tickets() {
                                    app.update_tickets(tickets);
                                }
                            }
                        }
                        KeyCode::Char('2') => {
                            if let Some(ticket) = app.tickets.get(app.selected_ticket) {
                                let _ = manager.update_status(&ticket.id, "in_progress");
                                if let Ok(tickets) = manager.list_tickets() {
                                    app.update_tickets(tickets);
                                }
                            }
                        }
                        KeyCode::Char('3') => {
                            if let Some(ticket) = app.tickets.get(app.selected_ticket) {
                                let _ = manager.update_status(&ticket.id, "closed");
                                if let Ok(tickets) = manager.list_tickets() {
                                    app.update_tickets(tickets);
                                }
                            }
                        }
                        KeyCode::Enter => {
                            if let Some(ticket) = app.tickets.get(app.selected_ticket) {
                                app.state = AppState::EditingTicket(app.selected_ticket);
                            }
                        }
                        _ => {}
                    },
                    AppState::Help => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('h') => {
                            app.state = AppState::Normal;
                        }
                        _ => {}
                    },
                    AppState::EditingTicket(_) => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            app.state = AppState::Normal;
                        }
                        _ => {}
                    },
                    AppState::CreatingTicket => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            app.state = AppState::Normal;
                        }
                        _ => {}
                    },
                }
            }
        }

        // Handle async events
        while let Ok(event) = rx.try_recv() {
            match event {
                AppEvent::Refresh => {
                    if let Ok(tickets) = manager.list_tickets() {
                        app.update_tickets(tickets);
                    }
                }
                AppEvent::Quit => break,
                _ => {}
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.size());

    // Header
    let header = Paragraph::new("Ticketr - TUI Mode")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, chunks[0]);

    // Main content
    match app.state {
        AppState::Normal => {
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
                .split(chunks[1]);

            // Ticket list
            let items: Vec<ListItem> = app
                .tickets
                .iter()
                .enumerate()
                .map(|(i, ticket)| {
                    let style = if i == app.selected_ticket {
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };

                    let status_color = match ticket.status.as_str() {
                        "open" => Color::Green,
                        "in_progress" => Color::Yellow,
                        "closed" => Color::Blue,
                        "blocked" => Color::Red,
                        "ready" => Color::Cyan,
                        "icebox" => Color::DarkGray,
                        "archive" => Color::DarkGray,
                        _ => Color::White,
                    };

                    ListItem::new(Line::from(vec![
                        Span::styled(format!("{} ", ticket.id), style),
                        Span::styled(
                            format!("[{}] ", ticket.status),
                            Style::default().fg(status_color)
                        ),
                        Span::styled(&ticket.title, style),
                    ]))
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Tickets"));
            f.render_stateful_widget(list, main_chunks[0], &mut app.list_state.clone());

            // Ticket details
            if let Some(ticket) = app.tickets.get(app.selected_ticket) {
                let details = vec![
                    Line::from(vec![
                        Span::styled("ID: ", Style::default().fg(Color::Cyan)),
                        Span::styled(&ticket.id, Style::default().add_modifier(Modifier::BOLD)),
                    ]),
                    Line::from(vec![
                        Span::styled("Title: ", Style::default().fg(Color::Cyan)),
                        Span::styled(&ticket.title, Style::default()),
                    ]),
                    Line::from(vec![
                        Span::styled("Status: ", Style::default().fg(Color::Cyan)),
                        Span::styled(&ticket.status, Style::default()),
                    ]),
                    Line::from(vec![
                        Span::styled("Priority: ", Style::default().fg(Color::Cyan)),
                        Span::styled(format!("{}", ticket.priority), Style::default()),
                    ]),
                    Line::from(vec![
                        Span::styled("Created: ", Style::default().fg(Color::Cyan)),
                        Span::styled(ticket.created.format("%Y-%m-%d %H:%M").to_string(), Style::default()),
                    ]),
                ];

                let mut details_text = details;
                if let Some(ref description) = ticket.description {
                    details_text.push(Line::from(""));
                    details_text.push(Line::from(vec![
                        Span::styled("Description:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    ]));
                    for line in description.lines().collect::<Vec<&str>>() {
                        details_text.push(Line::from(vec![
                            Span::styled("  ", Style::default()),
                            Span::styled(line, Style::default()),
                        ]));
                    }
                }

                let details_para = Paragraph::new(details_text)
                    .block(Block::default().borders(Borders::ALL).title("Details"))
                    .wrap(Wrap { trim: true });
                f.render_widget(details_para, main_chunks[1]);
            }
        }
        AppState::Help => {
            let help_text = vec![
                Line::from("Help - Keyboard Shortcuts:"),
                Line::from(""),
                Line::from("Navigation:"),
                Line::from("  j/Down    - Move down"),
                Line::from("  k/Up      - Move up"),
                Line::from("  h/Left    - Show help"),
                Line::from("  q/Esc     - Quit/Back"),
                Line::from(""),
                Line::from("Actions:"),
                Line::from("  Enter     - View ticket details"),
                Line::from("  r         - Refresh tickets"),
                Line::from("  1         - Set status to 'open'"),
                Line::from("  2         - Set status to 'in_progress'"),
                Line::from("  3         - Set status to 'closed'"),
                Line::from(""),
                Line::from("Press 'h', 'q', or 'Esc' to return"),
            ];

            let help_para = Paragraph::new(help_text)
                .block(Block::default().borders(Borders::ALL).title("Help"))
                .wrap(Wrap { trim: true });
            f.render_widget(help_para, chunks[1]);
        }
        AppState::EditingTicket(_) => {
            let edit_text = vec![
                Line::from("Ticket Details View"),
                Line::from(""),
                Line::from("Press 'q' or 'Esc' to return to list"),
            ];

            let edit_para = Paragraph::new(edit_text)
                .block(Block::default().borders(Borders::ALL).title("Ticket Details"))
                .wrap(Wrap { trim: true });
            f.render_widget(edit_para, chunks[1]);
        }
        AppState::CreatingTicket => {
            let create_text = vec![
                Line::from("Create New Ticket"),
                Line::from(""),
                Line::from("Press 'q' or 'Esc' to cancel"),
            ];

            let create_para = Paragraph::new(create_text)
                .block(Block::default().borders(Borders::ALL).title("Create Ticket"))
                .wrap(Wrap { trim: true });
            f.render_widget(create_para, chunks[1]);
        }
    }

    // Footer
    let footer_text = match app.state {
        AppState::Normal => {
            if let Some(ticket) = app.tickets.get(app.selected_ticket) {
                format!("{} | {} | Press 'h' for help", ticket.id, ticket.status)
            } else {
                "No tickets | Press 'h' for help".to_string()
            }
        }
        AppState::Help => "Help Mode | Press 'h', 'q', or 'Esc' to return".to_string(),
        AppState::EditingTicket(_) => "Ticket Details | Press 'q' or 'Esc' to return".to_string(),
        AppState::CreatingTicket => "Create Ticket | Press 'q' or 'Esc' to cancel".to_string(),
    };

    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[2]);
}
