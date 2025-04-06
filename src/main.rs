use crossterm::{
    cursor::{self, MoveTo},
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    queue,
    style::{self, Print, Color},
    terminal::{self, ClearType},
};
use std::io::{stdout, Write, Result};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

static SPINNER_FRAMES:&[&str]=&["⠋","⠙","⠹","⠸","⠼","⠴","⠦","⠧","⠇","⠏"];

// help content to display in the help window
static HELP_CONTENT: &[&str] = &[
    "Vbrute Help",
    "===========",
    "",
    "Help Navigation:",
    "  j/↓ - Scroll down",
    "  k/↑ - Scroll up",
    "  h/← - Scroll left",
    "  l/→ - Scroll right",
    "  q   - Close help window",
    "",
    "Commands:",
    "  :?                           - Show this help",
    "  :q                           - Quit application",
    "  :start                       - Start brute force simulation",
    "  :stop                        - Stop brute force simulation",
    "  :i <bin/blf> <path>          - Import database file",
    "  :m <seed/pkey/milksad> [n]   - Set search mode with optional word count",
    "  :bc <blockchain>             - Set target blockchain",
    "",
    "Supported blockchains:",
    "  btc, eth, bnb, xrp, doge, sol, ltc, bch, bsv",
    "",
    "Examples:",
    "  :i bin ~/path/to/db.bin      - Import binary database",
    "  :i blf ~/path/to/db.blf      - Import BLF database",
    "  :m seed 12                   - Set to 12-word seedphrase mode",
    "  :m milksad 25                - Set to 25-word Milk Sad mode",
    "  :m privkey                   - Set to random private keys mode",
    "  :bc btc                      - Set target blockchain to Bitcoin",
];

fn main() -> Result<()> {
    let mut stdout = stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, terminal::EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;
    let (_cols, rows) = terminal::size()?;

    let mut typing_command = false;
    let mut command_buffer = String::new();
    let mut status_text = "idle".to_string();
    let spinner_running = Arc::new(Mutex::new(false));
    let spinner_row = rows - 1;
    
    // help window state
    let mut showing_help = false;
    let mut help_scroll_y = 0;
    let mut help_scroll_x = 0;

    draw_welcome_screen(&mut stdout)?;

    loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                if showing_help {
                    // handle keys when help window is shown
                    match key_event.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            showing_help = false;
                            draw_welcome_screen(&mut stdout)?;
                        },
                        KeyCode::Char('j') | KeyCode::Down => {
                            if help_scroll_y < HELP_CONTENT.len().saturating_sub(10) {
                                help_scroll_y += 1;
                                draw_help_window(&mut stdout, help_scroll_y, help_scroll_x)?;
                            }
                        },
                        KeyCode::Char('k') | KeyCode::Up => {
                            if help_scroll_y > 0 {
                                help_scroll_y -= 1;
                                draw_help_window(&mut stdout, help_scroll_y, help_scroll_x)?;
                            }
                        },
                        KeyCode::Char('h') | KeyCode::Left => {
                            if help_scroll_x > 0 {
                                help_scroll_x -= 1;
                                draw_help_window(&mut stdout, help_scroll_y, help_scroll_x)?;
                            }
                        },
                        KeyCode::Char('l') | KeyCode::Right => {
                            help_scroll_x += 1;
                            draw_help_window(&mut stdout, help_scroll_y, help_scroll_x)?;
                        },
                        _ => {}
                    }
                } else {
                    // handle keys when help window is not shown
                    match key_event.code {
                        KeyCode::Char(':') if !typing_command => {
                            typing_command = true;
                            command_buffer.clear();
                        }
                        KeyCode::Char(c) if typing_command => {
                            command_buffer.push(c);
                        }
                        KeyCode::Backspace if typing_command => {
                            command_buffer.pop();
                        }
                        KeyCode::Enter if typing_command => {
                            if command_buffer == "?" {
                                // show help window
                                showing_help = true;
                                help_scroll_y = 0;
                                help_scroll_x = 0;
                                draw_help_window(&mut stdout, help_scroll_y, help_scroll_x)?;
                                typing_command = false;
                                command_buffer.clear();
                            } else if handle_command(&command_buffer, &mut status_text, spinner_running.clone(), spinner_row)? {
                                terminal::disable_raw_mode()?;
                                execute!(stdout, terminal::LeaveAlternateScreen)?;
                                return Ok(());
                            } else {
                                typing_command = false;
                                command_buffer.clear();
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        
        if !showing_help {
            draw_bottom(&mut stdout, &status_text, typing_command, &command_buffer)?;
        }
        stdout.flush()?;
    }
}

fn draw_help_window(stdout: &mut std::io::Stdout, scroll_y: usize, scroll_x: usize) -> Result<()> {
    let (cols, rows) = terminal::size()?;
    
    // calc dimensions for the help window
    let window_height = rows.saturating_sub(5) as usize;
    let window_width = cols.saturating_sub(4) as usize;
    let start_row = 2;
    let start_col = 2;
    
    // clear the screen
    queue!(
        stdout,
        terminal::Clear(ClearType::All),
    )?;
    
    // draw the help content
    for (i, line) in HELP_CONTENT.iter().skip(scroll_y).take(window_height).enumerate() {
        let display_line = if line.len() > scroll_x {
            &line[scroll_x.min(line.len())..line.len().min(scroll_x + window_width)]
        } else {
            ""
        };
        
        queue!(
            stdout,
            cursor::MoveTo(start_col, start_row as u16 + i as u16),
            style::Print(display_line),
        )?;
    }
    
    // draw the footer with instructions
    queue!(
        stdout,
        cursor::MoveTo(start_col, rows - 3),
        style::SetForegroundColor(Color::Yellow),
        style::Print("Press 'q' to close help, arrow keys or hjkl to scroll"),
        style::ResetColor,
    )?;
    
    // draw the status bar
    queue!(
        stdout,
        cursor::MoveTo(0, rows - 2),
        terminal::Clear(ClearType::CurrentLine),
        style::SetBackgroundColor(Color::White),
        style::SetForegroundColor(Color::Black),
        style::Print(" ".repeat(cols as usize)),
        style::ResetColor,
    )?;
    queue!(
        stdout,
        cursor::MoveTo(0, rows - 2),
        style::SetBackgroundColor(Color::White),
        style::SetForegroundColor(Color::Black),
        style::Print("Help"),
        style::ResetColor,
    )?;
    
    stdout.flush()?;
    Ok(())
}

fn start_spinner(running: Arc<Mutex<bool>>, status_row: u16) {
    thread::spawn(move || {
        let mut stdout = stdout();
        let mut i = 0;
        
        execute!(stdout, cursor::DisableBlinking).ok();

        while *running.lock().unwrap() {
            let (cols, _) = terminal::size().unwrap();
            let spinner = SPINNER_FRAMES[i % SPINNER_FRAMES.len()];

            queue!(
                stdout,
                cursor::SavePosition,
                cursor::MoveTo(cols - 2, status_row),
                Print(spinner),
                cursor::RestorePosition
            ).unwrap();
            stdout.flush().unwrap();

            i += 1;
            thread::sleep(Duration::from_millis(80));
        }

        let (cols, _) = terminal::size().unwrap();
        queue!(
            stdout,
            cursor::SavePosition,
            cursor::MoveTo(cols - 2, status_row),
            Print(" "),
            cursor::RestorePosition,
            cursor::EnableBlinking
        ).unwrap();
        stdout.flush().unwrap();
    });
}

fn draw_welcome_screen(stdout: &mut std::io::Stdout) -> Result<()> {
    let (cols, rows) = terminal::size()?;
    let middle_row = rows / 2 - 5;

    queue!(
        stdout,
        terminal::Clear(ClearType::All),
        cursor::MoveTo(cols / 2 - "Vbrute av0.1".len() as u16 / 2, middle_row),
        style::SetForegroundColor(Color::Yellow),
        style::Print("Vbrute av0.1"),
        style::ResetColor,
        cursor::MoveTo(cols / 2 - "Written in Rust by a legionary".len() as u16 / 2, middle_row + 1),
        style::Print("Written in Rust by a legionary"),
        cursor::MoveTo(cols / 2 - "type :?<Enter>   ..................   for help".len() as u16 / 2, middle_row + 3),
        style::Print("type :?<Enter>   ..................   for help"),
        cursor::MoveTo(cols / 2 - "type :q<Enter>   ..................    to exit".len() as u16 / 2, middle_row + 4),
        style::Print("type :q<Enter>   ...................   to exit"),
        cursor::MoveTo(cols / 2 - "type :start<Enter>   to start brute simulation".len() as u16 / 2, middle_row + 6),
        style::Print("type :start<Enter>   to start brute simulation"),
        cursor::MoveTo(cols / 2 - "type :stop<Enter>   t  o stop brute simulation".len() as u16 / 2, middle_row + 7),
        style::Print("type :stop<Enter>     to stop brute simulation"),
    )?; 
    stdout.flush()?;
    Ok(())
}

fn draw_bottom(
    stdout: &mut std::io::Stdout,
    status_text: &str,
    typing_command: bool,
    command_buffer: &str,
) -> Result<()> {
    let (cols, rows) = terminal::size()?;

    queue!(
        stdout,
        cursor::MoveTo(0, rows - 2),
        terminal::Clear(ClearType::CurrentLine),
        style::SetBackgroundColor(Color::White),
        style::SetForegroundColor(Color::Black),
        style::Print(" ".repeat(cols as usize)),
        style::ResetColor,
    )?;
    queue!(
        stdout,
        cursor::MoveTo(0, rows - 2),
        style::SetBackgroundColor(Color::White),
        style::SetForegroundColor(Color::Black),
        style::Print(status_text),
        style::ResetColor,
    )?;
    queue!(
        stdout,
        cursor::MoveTo(0, rows - 1),
        terminal::Clear(ClearType::CurrentLine),
    )?;
    if typing_command {
        queue!(
            stdout,
            cursor::MoveTo(0, rows - 1),
            style::Print(format!(":{}", command_buffer)),
        )?;
    }
    Ok(())
}

fn handle_command(command: &str, status_text: &mut String, spinner_running: Arc<Mutex<bool>>, spinner_row: u16) -> Result<bool> {
    // Split command into parts for commands that need arguments
    let parts: Vec<&str> = command.split_whitespace().collect();
    
    // Static variables to store database paths and settings
    static mut DBIN: Option<String> = None;
    static mut DBLF: Option<String> = None;
    static mut MODE: Option<String> = None;
    static mut MODE_WORD_COUNT: usize = 0;
    static mut BLOCKCHAIN: Option<String> = None;

    match parts.get(0).map(|s| *s) {
        Some("?") => {
            // Help is now handled in the main loop
            return Ok(false);
        }
        Some("start") => {
            *status_text = "Starting...".to_string();
            let mut running = spinner_running.lock().unwrap();
            if !*running {
                *running = true;
                start_spinner(spinner_running.clone(), spinner_row);
            }
            drop(running);

            thread::spawn({
                let spinner_running = spinner_running.clone();
                let _status_text = Arc::new(Mutex::new(status_text.clone()));
                move || {
                    thread::sleep(Duration::from_secs(5));
                    *spinner_running.lock().unwrap() = false;
                }
            });
        }
        Some("stop") => {
            *status_text = "Stopping...".to_string();
            *spinner_running.lock().unwrap() = false;
        }
        Some("i") => {
            if parts.len() < 3 {
                *status_text = "Usage: :i <bin/blf> <path>".to_string();
            } else {
                let db_type = parts[1];
                let path = parts[2..].join(" ");
                
                // Check if file exists
                if std::path::Path::new(&path).exists() {
                    unsafe {
                        match db_type {
                            "bin" => {
                                DBIN = Some(path.clone());
                                *status_text = format!("Imported {}", path);
                            },
                            "blf" => {
                                DBLF = Some(path.clone());
                                *status_text = format!("Imported {}", path);
                            },
                            _ => {
                                *status_text = "Usage: :i <bin/blf> <path>".to_string();
                            }
                        }
                    }
                } else {
                    *status_text = format!("Cannot import {}: inexistent", path);
                }
            }
        }
        Some("m") => {
            if parts.len() < 2 {
                *status_text = "Usage: :m <seed/pkey/milksad> [word count]".to_string();
            } else {
                let mode = parts[1].to_lowercase();
                
                unsafe {
                    match mode.as_str() {
                        "seed" => {
                            let word_count = if parts.len() > 2 {
                                parts[2].parse::<usize>().unwrap_or(12)
                            } else {
                                12
                            };
                            
                            MODE = Some("seed".to_string());
                            MODE_WORD_COUNT = word_count;
                            *status_text = format!("Search mode set: {}-word seedphrase", word_count);
                        },
                        "milksad" => {
                            let word_count = if parts.len() > 2 {
                                parts[2].parse::<usize>().unwrap_or(25)
                            } else {
                                25
                            };
                            
                            MODE = Some("milksad".to_string());
                            MODE_WORD_COUNT = word_count;
                            *status_text = format!("Search mode set: {}-word seedphrase (Milk Sad)", word_count);
                        },
                        "pkey" | "privkey" => {
                            MODE = Some("privkey".to_string());
                            MODE_WORD_COUNT = 0;
                            *status_text = "Search mode set: Random private keys".to_string();
                        },
                        _ => {
                            *status_text = "Usage: :m <seed/pkey/milksad> [word count]".to_string();
                        }
                    }
                }
            }
        }
        Some("bc") => {
            if parts.len() < 2 {
                *status_text = "Usage: :bc <btc/eth/bnb/xrp/doge/sol/ltc/bch/bsv>".to_string();
            } else {
                let blockchain = parts[1].to_lowercase();
                let valid_blockchains = ["btc", "eth", "bnb", "xrp", "doge", "sol", "ltc", "bch", "bsv"];
                
                if valid_blockchains.contains(&blockchain.as_str()) {
                    unsafe {
                        BLOCKCHAIN = Some(blockchain.clone());
                    }
                    *status_text = format!("Target blockchain set: {}", blockchain.to_uppercase());
                } else {
                    *status_text = "Usage: :bc <btc/eth/bnb/xrp/doge/sol/ltc/bch/bsv>".to_string();
                }
            }
        }
        Some("q") => {
            return Ok(true);
        }
        _ => {
            *status_text = format!("Unknown command: {}", command);
        }
    }
    Ok(false)
}