use crossterm::{
    cursor::{self, MoveTo},
    event::{self, Event, KeyCode},
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

    draw_welcome_screen(&mut stdout)?;

    loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
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
                        if handle_command(&command_buffer, &mut status_text, spinner_running.clone(), spinner_row)? {
                            terminal::disable_raw_mode()?;
                            execute!(stdout, terminal::LeaveAlternateScreen)?;
                            return Ok(());
                        }
                        typing_command = false;
                        command_buffer.clear();
                    }
                    _ => {}
                }
            }
        }
        draw_bottom(&mut stdout, &status_text, typing_command, &command_buffer)?;
        stdout.flush()?;
    }
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
    Ok(())}

fn handle_command(command: &str, status_text: &mut String, spinner_running: Arc<Mutex<bool>>, spinner_row: u16) -> Result<bool> {
    match command {
        "?" => {
            *status_text = "Commands: :start, :stop, :q".to_string();
        }
        "start" => {
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
        "stop" => {
            *status_text = "Stopping...".to_string();
            *spinner_running.lock().unwrap() = false;
        }
        "q" => {
            return Ok(true);
        }
        _ => {
            *status_text = format!("Unknown command: {}", command);
        }
    }
    Ok(false)
}