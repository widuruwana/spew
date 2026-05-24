/* ---> Diary Entry #1 <---
  This project is my first time learning how to use Rust so you will
  see blocks of comments like these that either overly explain code
  or vent my personal frustrations, hopefull as less as possible.
*/

/* ---> The Imports
  VecDeque is a double-ended queue data structure
  BufRead is what Rust calls a Trait (kind of like an interface/contract).
  BufRead unlocks the ability to read Streams line-by-line.
  Arc (Atomic Reference Counter) used to share data safely across multiple CPU threads.
  RwLock (Read-Write Lock) Allows multiple threads to read data but only one to write/modify.
  thread: the core library used to spawn and manage background worker threads.
  serde_json::Value is a third party tool used to parse and handle raw JSON
*/
use std::collections::VecDeque;
use std::io::{self, BufRead};
use std::sync::{Arc, RwLock};
use std::thread;
use serde_json::Value;

use ratatui::{
    // ratatui needs a backend driver to talk to terminal (which is Crossterm)
    backend::CrosstermBackend,
    // ratatui divides the screen into rectangular chunks, layout is how we define
    //-those chunks. ( Its low-key like CSS flexbox )
    layout::{Constraint, Direction, Layout},
    // How we color and bold text
    style::{Color, Modifier, Style},
    // UI widgets. Block is a container with optional borders. List holds the log lines.
    widgets::{Block, Borders, List, ListItem},
    Terminal
};

use crossterm::{
    // How you read keyboard input, every keypress comes in as an event
    event::{self, Event, KeyCode},
    execute,
    // enable/disable_raw_mode makes every single keypress available instantly (normally termi-
    //-nal buffers keypresses until we hit enter). This is essential for TUI
    // Terminals have two screens, normal one is where we type commands, and alternate one is
    //-used by appls like vim. We switch into alternate screen so spew dont trash our terminal
    //-history. we then swtich back when we quit.
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}
};

// _ is a visual seperator like a comma
const BUFFER_CAPACITY: usize = 10_000;

#[derive(Clone)] // Tell rust to automatically write the code required to dup/cpy this struct.
struct LogEntry {
    ts: String,     // stroes timestamp
    level: String,  // stores the log severity level (ex: "INFO", "ERROR")
    msg: String,    // stores the log message text
    raw: String     // stores unparsed raw log line as fallback
}

fn parse_line(line: &str) -> LogEntry {
    // ::<serde_json::value> is called the "turbofish" syntax.
    // It tells the parser exactly what memory layout to use. In here we tells it to
    // parse the string to an unstrauctured, generic JSON tree (a Value).
    match serde_json::from_str::<Value>(line){
        Ok(json) => LogEntry {
            // .unwrap_or() is an alternative to .unwrap()
            // in here instead of crashing it gives the fallback string "???"
            ts:     json["ts"].as_str().unwrap_or("").to_string(),
            level:  json["level"].as_str().unwrap_or("").to_string(),
            msg:    json["msg"].as_str().unwrap_or("").to_string(),
            raw:    line.to_string(),
        },
        Err(_) => LogEntry {
            ts:     String::new(),
            level:  String::new(),
            msg:    String::new(),
            raw:    line.to_string(),
        },
    }
}

struct RingBuffer {
    entries: VecDeque<LogEntry>,
    capacity: usize
}

// impl -> implements a function to create this struct in memory
impl RingBuffer {
    // constructor
    fn new(capacity: usize) -> Self {
        Self {
            // preallocates memory for the queue so it doesnt have to slow down and
            //-resize itself later
            entries: VecDeque::with_capacity(capacity),
            // short for capacity: capacity. because the variable name matches the field
            //-name, rust let us write it only once.
            capacity 
        }
    }

    // &mut self tells rust this function needs exclusive, modifiable access to buffer
    fn push(&mut self, entry: LogEntry){
        // if buffer is full, delete the oldest entry to make room
        if self.entries.len() == self.capacity {
            self.entries.pop_front();
        }
        // inserts new log entry at the back of the queue
        self.entries.push_back(entry);
    }

    // &self tells rust it only reads data, returns a list of pointer to the logs
    fn filtered(&self, query: &str)-> Vec<&LogEntry> {
        // if no search term, instantly bundles all logs into a list and returns
        if query.is_empty(){
            return self.entries.iter().collect();
        }
        self.entries
            .iter()
            // iterates through the every log and check if it contains search phrase and keeps em
            .filter(|e| e.raw.contains(query))
            .collect()
    }

}

// if the terminal environment cannot be initialized, it returns an error
fn main() -> Result<(), Box<dyn std::error::Error>>{
    let buffer = Arc::new(RwLock::new(RingBuffer::new(BUFFER_CAPACITY)));
    let buffer_writer = Arc::clone(&buffer);

    // Ingestion thread
    // move is a strict rust keyword. Forces the Ingestion thread to take full ownership
    //-of buffer_writer so the main thread cant accidently delete it while background
    //-thread is still using it.
    thread::spawn(move || {
        let stdin = io::stdin();
        for line in stdin.lock().lines(){
            // .lines() iterator returns Result<String, Error>
            // .unwrap() tells the compiler that it expect this to succeed but in a case
            // -of failing to read this memory address or stream, instantly crash(panic)
            // the program right here.
            let line = line.unwrap();
            let entry = parse_line(&line);
            buffer_writer.write().unwrap().push(entry);
        }
    });

    // Setup terminal
    enable_raw_mode()?; // Now key presses are immediate
    let mut stdout = io::stdout(); // grab a handler to stdout

    // execute! is a macro that sends commands to the terminal and is used to switch
    //-screens and restore them.
    execute!(stdout, EnterAlternateScreen)?; // switch to alt screen
    let backend = CrosstermBackend::new(stdout); // wrap stdout in Crossterm backend 
    let mut terminal = Terminal::new(backend)?; // Wrap that in ratatui terminal
    // ^ this is also the object we called .draw() on

    let mut query = String::new();  // The current search string the user is typing
    let mut typing = false;         // Whether the filter bar is active  
    let mut frozen = false;         // Whether the viewpoint is paused.
    let mut scroll: usize = 0;      // Which index is currently selected/highlighted

    // print loop
    loop{
        // Draw
        {
            let buf = buffer.read().unwrap();
            let entries = buf.filtered(&query);
            let total = entries.len();

            // Auto scroll to bottom unless frozen
            if !frozen && total > 0 {
                // saturating_sub(1) is "total - 1" but safe. If tot = 0, it wont go below 0.
                scroll = total.saturating_sub(1);
            }

            // Iterates every log entry. enumerate() gives both index i and entry e
            let items: Vec<ListItem> = entries
                .iter()
                .enumerate()
                .map(|(i, e)| {
                    // Non JSON lines are shown raw. Otherwise format it as the clean table row.
                    let line = if e.ts.is_empty() {
                        e.raw.clone()
                    } else {
                        format!("[{}] {:5} | {}", e.ts, e.level, e.msg)
                    };

                    // Errors are Red and BOLD.
                    // Warning are Yellow.
                    // Everything else is Gray.
                    let style = if e.level == "error" || e.level == "ERROR" {
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                    } else if e.level == "warn" || e.level == "WARN" {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::Gray)
                    };

                    // current selected line gets REVERSED which means foreground and
                    //-background colors flips, making it look highlighted regardless of the
                    //-color scheme.
                    let selected = i == scroll;
                    let style = if selected {
                        style.add_modifier(Modifier::REVERSED)
                    } else {
                        style
                    };

                    // "ListItem::new(line)" Wraps the formatted string into a ListItem widget.
                    // Ratatui's List widget doesnt accept raw strings directly, it only 
                    //-accepts ListItems.
                    // ".style(style) applies the color and modifer that was decided earlier.
                    ListItem::new(line).style(style)
                }) // closes .map()
                .collect(); // .map() produces a lazy iterator, it havent done anything yet.
                            // .collect() is what forces it to run and gathers all the resulting
                            //-ListItems into a Vec<ListItem> that ratatui can use.

            // three possible status bar status.
            // typing -> shows current query with / prefix like vim
            // when frozen or live, show keybinding hints
            let status = if typing {
                format!("/ {}", query)
            } else if frozen {
                format!("FROZEN | q: quit  /: filter  Space: unfreeze  ↑↓: scroll")
            } else {
                format!("LIVE   | q: quit  /: filter  Space: freeze    ↑↓: scroll")
            };

            // "terminal.draw()" takes a closure that receives f(frame)
            terminal.draw(|f| {
                // Layout splits the screen vertically into two chunks.
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    // Min(1) -> Log list takes all available space
                    // Length(1) -> Status bar is only 1 line tall
                    .constraints([Constraint::Min(1), Constraint::Length(1)])
                    .split(f.area());

                // Creates the list widget from the items and renders it into the top chunk
                let list = List::new(items)
                    .block(Block::default().borders(Borders::NONE));
                f.render_widget(list, chunks[0]);

                // Renders the status bar as a Paragraph into the bottom chunk.
                // Black text on white background
                let status_widget = ratatui::widgets::Paragraph::new(status)
                    .style(Style::default().fg(Color::Black).bg(Color::White));
                f.render_widget(status_widget, chunks[1]);
            })?;
        }

        // Input
        // Screen refereshes at minimum every 100ms
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') if !typing => {  // "q" -> quit
                        break;                           
                    }
                    KeyCode::Char(' ') if !typing => {  // "spacebar" -> freeze
                        frozen = !frozen;
                    }
                    KeyCode::Char('/') if !typing => {  // "/" -> fileter bar
                        typing = true;
                    }
                    KeyCode::Esc => {                   // "Esc" -> exits typing mode
                        typing = false;                 //-And clears the search.
                        query.clear();
                    }
                    KeyCode::Enter => { 
                        typing = false;
                    }
                    KeyCode::Backspace if typing => {
                        query.pop();
                    }
                    KeyCode::Char(c) if typing => {
                        query.push(c);
                    }
                    // Scrolling up autmatically freezes the viewport
                    KeyCode::Up => {
                        frozen = true;
                        scroll = scroll.saturating_sub(1);
                    }
                    KeyCode::Down => {
                        // Graphs the read lock on the ring buffer to check how many entries
                        //-currently exist.
                        let buf = buffer.read().unwrap();
                        // Runs the current search query and counts how many lines match.
                        //-(Total number of lines visible on the screen)
                        let total = buf.filtered(&query).len();
                        // Checks if there is actually a line below the current one
                        if scroll + 1 < total {
                            scroll += 1; // Moves selection one line down
                        }
                    }
                    _ => {} // catch-all pattern in Rust match statements.
                }
            }
        }
    }       

    // Cleanup Terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
