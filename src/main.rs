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

// Serde is the standard Rust Serialization library. Deserialize is a trait that lets Rust
//-Automatically read a TOML file and map its content into a Rust struct.
use serde::Deserialize;

// Gruvbox dark color palette
const GRB_RED:     Color = Color::Rgb(204, 36,  29);
const GRB_YELLOW:  Color = Color::Rgb(215, 153, 33);
const GRB_GRAY:    Color = Color::Rgb(146, 131, 116);
const GRB_DIMGRAY: Color = Color::Rgb(80,  73,  69);
const GRB_WHITE:   Color = Color::Rgb(235, 219, 178);
const GRB_BG:      Color = Color::Rgb(40,  40,  40);
const GRB_FG:      Color = Color::Rgb(235, 219, 178);

// Struct directly maps to the [colors] section in the user's config.toml.
// #[derive(...)] tells Serde to automatically generate the code that reads the TOML file
//-into this struct.
#[derive(Deserialize)]
struct ColorConfig {
    error: Option<String>,
    warn:  Option<String>,
    info:  Option<String>,
    dim:   Option<String>,
}

#[derive(Deserialize)]
struct Config {
    colors: Option<ColorConfig>,
}

// Theme stores resolved Color values ready to be passed directly into Ratatui
struct Theme {
    error: Color,
    warn:  Color,
    info:  Color,
    dim:   Color,
}
 
impl Theme {
    // Default contructor, returns a theme with Gruvbox constatnts baked in.
    fn default() -> Self {
        Self {
            error: GRB_RED,
            warn:  GRB_YELLOW,
            info:  GRB_GRAY,
            dim:   GRB_DIMGRAY,
        }
    }

    // Start with default and attempts to override.
    fn load() -> Self {
        let mut theme = Theme::default();

        let config_path = dirs::home_dir()
            .map(|h| h.join(".config/spew/config.toml"));

        if let Some(path) = config_path {
            if let Ok(contents) = std::fs::read_to_string(&path) {
                if let Ok(config) = toml::from_str::<Config>(&contents) {
                    if let Some(colors) = config.colors {
                        if let Some(hex) = colors.error {
                            theme.error = hex_to_color(&hex).unwrap_or(theme.error);
                        }
                        if let Some(hex) = colors.warn {
                            theme.warn  = hex_to_color(&hex).unwrap_or(theme.warn);
                        }
                        if let Some(hex) = colors.info {
                            theme.info  = hex_to_color(&hex).unwrap_or(theme.info);
                        }
                        if let Some(hex) = colors.dim {
                            theme.dim   = hex_to_color(&hex).unwrap_or(theme.dim);
                        }
                    }
                }
            }
        }

        theme
    }
}

// ex: Converts a hex color string like #cc241d" into a ratatui Color:Rgb value
fn hex_to_color(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }

    // slices string into three pairs (red, green, blue) and parse each pair as a 
    //-base-16 number into a u8(0-255)
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(Color::Rgb(r, g, b))
}

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

const CONTEXT_LINES: usize = 5;
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

    // context: usize -> number of lines to show before and after each match
    // Returns a tuple of three things per entry.
    //      |> usize -> Original index
    //      |> bool -> is an actual match or just surronding context
    //      |> &LogEntry -> Entry itself
    fn filtered(&self, query: &str, context: usize)-> Vec<(usize, bool, &LogEntry)> {
        if query.is_empty(){
            return self.entries
                .iter()
                .enumerate()
                // false here means dont highlight anything
                .map(|(i, e)| (i, false, e))
                .collect();
        }
        
        let len = self.entries.len(); // total no. of entries in the buffer
        
        // BTreeSet will hold every line index we want to display, both matches and
        //-their surronding context lines. Also Automatically deduplicates and keep things sorted
        let mut indices = std::collections::BTreeSet::new();
        // Hashset tracking only the indices that actually matched the query
        // Keep to highlight the real match and keep the context lines dim
        let mut matched = std::collections::HashSet::new();

        // Loop through every entry log, if entries raw text contain search query its a match
        for (i, e) in self.entries.iter().enumerate() {
            if e.raw.contains(query) {
                // Record the index as a real match in matched HashSet
                matched.insert(i);
                // Calculate the window of lines to show around this match.
                // start is context lines before the match, saturating_sub prevent going < 0.
                // end is context lines after the match. .min(len) prevents going > End of Buffer
                let start = i.saturating_sub(context);
                let end = (i + context + 1).min(len);
                for j in start..end {
                    // Add every index in that window to BTreeSet
                    indices.insert(j);
                }
            }
        }

        // Takes the final sorted, deduplicated set of indices and build the return value
        // for each index i,
        //      i -> the index
        //      matched.contains(&i) -> true if actual match, false if context
        //      &self.entries[i] -> actual log entry at that position
        indices
            .iter()
            .map(|&i| (i, matched.contains(&i), &self.entries[i]))
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
    // Option<usize> means either None (no line expanded) or Some(i) (line i is expanded),
    //-this is how to track which line the panel is showing.
    let mut expanded: Option<usize> = None;

    // view_offset is the index of the first visible line.
    // The viewport only renders from view_offset to view_offset + visible_height
    let mut view_offset: usize = 0;

    // loading the theme
    let theme = Theme::load();

    // print loop
    loop{
        // Draw
        {
            let buf = buffer.read().unwrap();
            let entries = buf.filtered(&query, CONTEXT_LINES);
            let total = entries.len();
    
            if !frozen && total > 0 {
                scroll = total.saturating_sub(1);
            }

            let term_height = terminal.size()?.height as usize;
            // When expanded the log list only gets half the screen
            let list_height = if expanded.is_some() {
                term_height / 2
            } else {
                term_height.saturating_sub(1) // minus status bar
            };

            // Keep view_offset in sync so scroll is always visible
            if scroll < view_offset {
                view_offset = scroll;
            } else if scroll >= view_offset + list_height {
                view_offset = scroll.saturating_sub(list_height - 1);
            }

            let visible_entries: Vec<_> = entries
                .iter()
                .skip(view_offset)
                .take(list_height)
                .collect();
    
            let items: Vec<ListItem> = visible_entries
                .iter()
                .map(|(idx, is_match, e)| {
                    let line = if e.ts.is_empty() {
                        e.raw.clone()
                    } else {
                        format!("[{}] {:5} | {}", e.ts, e.level, e.msg)
                    };

                    let style = if !is_match && !query.is_empty() {
                        Style::default().fg(theme.dim)
                    } else if e.level == "error" || e.level == "ERROR" {
                        Style::default().fg(theme.error).add_modifier(Modifier::BOLD)
                    } else if e.level == "warn" || e.level == "WARN" {
                        Style::default().fg(theme.warn)
                    } else {
                        Style::default().fg(theme.info)
                    };
    
                    let selected = *idx == scroll;
                    let style = if selected {
                        style.add_modifier(Modifier::REVERSED)
                    } else {
                        style
                    };
        
                    ListItem::new(line).style(style)
                })
                .collect();

            let status = if typing {
                format!("/ {}", query)
            } else if frozen {
                if expanded.is_some() {
                    format!("FROZEN | q: quit  /: filter  Space: unfreeze  ↑↓: scroll  Enter: collapse")
                } else {
                    format!("FROZEN | q: quit  /: filter  Space: unfreeze  ↑↓: scroll  Enter: expand")
                }
            } else {
                if expanded.is_some() {
                    format!("LIVE   | q: quit  /: filter  Space: freeze    ↑↓: scroll  Enter: collapse")
                } else {
                    format!("LIVE   | q: quit  /: filter  Space: freeze    ↑↓: scroll  Enter: expand")
                }
            };

            // Find the expanded entry using scroll position in the filtered list
            let expanded_detail = if let Some(_) = expanded {
                entries.get(scroll).map(|(_, _, e)| {
                    match serde_json::from_str::<Value>(&e.raw) {
                        Ok(json) => serde_json::to_string_pretty(&json)
                            .unwrap_or(e.raw.clone()),
                        Err(_) => e.raw.clone(),
                    }
                })
            } else {
                None
            };

            terminal.draw(|f| {
                let constraints = if expanded.is_some() {
                    vec![
                        Constraint::Percentage(50),
                        Constraint::Percentage(50),
                        Constraint::Length(1),
                    ]
                } else {
                    vec![Constraint::Min(1), Constraint::Length(0), Constraint::Length(1)]
                };

                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(constraints)
                    .split(f.area());

                if items.is_empty() && !query.is_empty() {
                    let empty = ratatui::widgets::Paragraph::new("No results for your search.")
                        .style(Style::default().fg(GRB_DIMGRAY).bg(GRB_BG));
                    f.render_widget(empty, chunks[0]);
                } else {
                    let list = List::new(items)
                        .block(Block::default().borders(Borders::NONE)
                        .style(Style::default().bg(GRB_BG)));
                    f.render_widget(list, chunks[0]);
                }

                if let Some(detail) = expanded_detail {
                    let panel = ratatui::widgets::Paragraph::new(detail)
                        .block(Block::default().borders(Borders::ALL).title(" Detail ")
                        .style(Style::default().bg(GRB_BG).fg(GRB_GRAY)))
                        .style(Style::default().fg(GRB_WHITE).bg(GRB_BG));
                    f.render_widget(panel, chunks[1]);
                }
        
                let status_widget = ratatui::widgets::Paragraph::new(status)
                    .style(Style::default().fg(GRB_BG).bg(GRB_FG));
                f.render_widget(status_widget, chunks[2]);
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
                    KeyCode::Enter if !typing=> { 
                        if expanded.is_some() {
                            expanded = None;
                        } else {
                            expanded = Some(scroll);
                        }
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
                        let total = buf.filtered(&query, CONTEXT_LINES).len();
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
