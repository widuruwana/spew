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
const GRB_BG0_H:  Color = Color::Rgb(29,  32,  33);   // darkest bg
const GRB_BG0:    Color = Color::Rgb(40,  40,  40);   // main bg
const GRB_BG1:    Color = Color::Rgb(60,  56,  54);   // slightly lighter bg
const GRB_BG2:    Color = Color::Rgb(80,  73,  69);   // selection bg
const GRB_BG3:    Color = Color::Rgb(102, 92,  84);   // inactive
const GRB_BG4:    Color = Color::Rgb(124, 111, 100);  // comments
const GRB_FG0:    Color = Color::Rgb(251, 241, 199);  // brightest fg
const GRB_FG1:    Color = Color::Rgb(235, 219, 178);  // main fg
const GRB_FG2:    Color = Color::Rgb(213, 196, 161);  // dimmer fg
const GRB_FG3:    Color = Color::Rgb(189, 174, 147);  // even dimmer
const GRB_FG4:    Color = Color::Rgb(168, 153, 132);  // dimmest fg
const GRB_RED:    Color = Color::Rgb(204, 36,  29);   // error
const GRB_RED_L:  Color = Color::Rgb(251, 73,  52);   // bright red
const GRB_GREEN:  Color = Color::Rgb(152, 151, 26);   // success
const GRB_GREEN_L:Color = Color::Rgb(184, 187, 38);   // bright green
const GRB_YELLOW: Color = Color::Rgb(215, 153, 33);   // warn
const GRB_YELLOW_L:Color= Color::Rgb(250, 189, 47);   // bright yellow
const GRB_BLUE:   Color = Color::Rgb(69,  133, 136);  // db/query
const GRB_BLUE_L: Color = Color::Rgb(131, 165, 152);  // bright blue
const GRB_PURPLE: Color = Color::Rgb(177, 98,  134);  // auth
const GRB_PURPLE_L:Color = Color::Rgb(211, 134, 155); // bright purple
const GRB_AQUA:   Color = Color::Rgb(104, 157, 106);  // ok/success
const GRB_AQUA_L: Color = Color::Rgb(142, 192, 124);  // bright aqua
const GRB_ORANGE: Color = Color::Rgb(214, 93,  14);   // warning/retry
const GRB_ORANGE_L:Color = Color::Rgb(254, 128, 25);  // bright orange

// Struct directly maps to the [colors] section in the user's config.toml.
// #[derive(...)] tells Serde to automatically generate the code that reads the TOML file
//-into this struct.
#[derive(Deserialize)]
struct ColorConfig {
    error: Option<String>,
    warn:  Option<String>,
    info:  Option<String>,
    dim:   Option<String>,
    db:    Option<String>,
    auth:  Option<String>,
    conn:  Option<String>,
    ok:    Option<String>,
}

#[derive(Deserialize)]
struct Config {
    colors: Option<ColorConfig>,
}

// Theme stores resolved Color values ready to be passed directly into Ratatui
struct Theme {
    error:  Color,
    warn:   Color,
    info:   Color,
    dim:    Color,
    db:     Color,
    auth:   Color,
    conn:   Color,
    ok:     Color,
}
 
impl Theme {
    // Default contructor, returns a theme with Gruvbox constatnts baked in.
    fn default() -> Self {
        Self {
            error: GRB_RED_L,
            warn:  GRB_YELLOW_L,
            info:  GRB_FG3,
            dim:   GRB_BG4,
            db:    GRB_BLUE_L,
            auth:  GRB_PURPLE_L,
            conn:  GRB_ORANGE_L,
            ok:    GRB_FG3,
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
                        if let Some(hex) = colors.error{
                            theme.error = hex_to_color(&hex).unwrap_or(theme.error);
                        }
                        if let Some(hex) = colors.warn{
                            theme.warn = hex_to_color(&hex).unwrap_or(theme.warn);
                        }
                        if let Some(hex) = colors.info{
                            theme.info = hex_to_color(&hex).unwrap_or(theme.info);
                        }
                        if let Some(hex) = colors.dim{
                            theme.dim = hex_to_color(&hex).unwrap_or(theme.dim);
                        }
                        if let Some(hex) = colors.db{
                            theme.db = hex_to_color(&hex).unwrap_or(theme.db);
                        }
                        if let Some(hex) = colors.auth{
                            theme.auth = hex_to_color(&hex).unwrap_or(theme.auth);
                        }
                        if let Some(hex) = colors.conn{
                            theme.conn = hex_to_color(&hex).unwrap_or(theme.conn);
                        }
                        if let Some(hex) = colors.ok{
                            theme.ok = hex_to_color(&hex).unwrap_or(theme.ok);
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
    id: usize,      // Unique Identifier
    ts: String,     // stroes timestamp
    level: String,  // stores the log severity level (ex: "INFO", "ERROR")
    msg: String,    // stores the log message text
    raw: String     // stores unparsed raw log line as fallback
}

fn parse_line(line: &str, id: usize) -> LogEntry {
    // ::<serde_json::value> is called the "turbofish" syntax.
    // It tells the parser exactly what memory layout to use. In here we tells it to
    // parse the string to an unstrauctured, generic JSON tree (a Value).
    match serde_json::from_str::<Value>(line){
        Ok(json) => LogEntry {
            id,
            // .unwrap_or() is an alternative to .unwrap()
            // in here instead of crashing it gives the fallback string "???"
            ts:     json["ts"].as_str().unwrap_or("").to_string(),
            level:  json["level"].as_str().unwrap_or("").to_string(),
            msg:    json["msg"].as_str().unwrap_or("").to_string(),
            raw:    line.to_string(),
        },
        Err(_) => LogEntry {
            id,
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
        let mut next_id: usize = 0;

        for line in stdin.lock().lines(){
            // .lines() iterator returns Result<String, Error>
            // .unwrap() tells the compiler that it expect this to succeed but in a case
            // -of failing to read this memory address or stream, instantly crash(panic)
            // the program right here.
            let line = line.unwrap();
            let entry = parse_line(&line, next_id);
            next_id = next_id.wrapping_add(1);

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
    let mut selected_id: Option<usize> = None; // Tracks the exact log we are locked onto
    // Option<usize> means either None (no line expanded) or Some(i) (line i is expanded),
    //-this is how to track which line the panel is showing.
    
    let mut frozen_entries: Vec<(usize, bool, LogEntry)> = Vec::new();
    let mut expanded: bool = false;

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

            let entries: Vec<(usize, bool, &LogEntry)> = if frozen {
                frozen_entries.iter().map(|(i, m, e)| (*i, *m, e)).collect()
            } else {
                buf.filtered(&query, CONTEXT_LINES)
            };

            let total = entries.len();

            let term_height = terminal.size()?.height as usize;
            // When expanded the log list only gets half the screen
            let list_height = if expanded {
                term_height / 2
            } else {
                term_height.saturating_sub(1) // minus status bar
            };

            if !frozen {
                // Live mode: chase the bottom
                if total > 0 {
                    scroll = total.saturating_sub(1);
                    // Always record the ID of the newest log so if we freeze, we anchor to it.
                    selected_id = Some(entries[scroll].2.id);
                }
                view_offset = scroll.saturating_sub(list_height.saturating_sub(1));
            }else {
                // Keep view_offset clamped around our newly calculated scroll position
                if scroll < view_offset {
                    view_offset = scroll;
                } else if scroll >= view_offset + list_height {
                    view_offset = scroll.saturating_sub(list_height.saturating_sub(1));
                }
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
                        Style::default().fg(theme.error).bg(GRB_BG1).add_modifier(Modifier::BOLD)
                    } else if e.level == "warn" || e.level == "WARN" {
                        Style::default().fg(theme.warn).bg(GRB_BG0)
                    } else if e.msg.contains("db") || e.msg.contains("query") || e.msg.contains("sql") {
                        Style::default().fg(theme.db)
                    } else if e.msg.contains("auth") || e.msg.contains("login") || e.msg.contains("token") {
                        Style::default().fg(theme.auth)
                    } else if e.msg.contains("connect") || e.msg.contains("retry") || e.msg.contains("timeout") {
                        Style::default().fg(theme.conn)
                    } else if e.msg.contains("ok") || e.msg.contains("success") || e.msg.contains("handled") {
                        Style::default().fg(theme.ok)
                    } else {
                        Style::default().fg(theme.info)
                    };

                    let selected = *idx == scroll;
                    let style = if selected {
                        style.add_modifier(Modifier::BOLD).add_modifier(Modifier::REVERSED)
                    } else {
                        style
                    };
        
                    ListItem::new(line).style(style)
                })
                .collect();

            let status = if typing {
                format!("/ {}", query)
            } else if frozen {
                if expanded {
                    format!("FROZEN | q: quit  /: filter  Space: unfreeze  ↑↓: scroll  Enter: collapse")
                } else {
                    format!("FROZEN | q: quit  /: filter  Space: unfreeze  ↑↓: scroll  Enter: expand")
                }
            } else {
                if expanded {
                    format!("LIVE   | q: quit  /: filter  Space: freeze    ↑↓: scroll  Enter: collapse")
                } else {
                    format!("LIVE   | q: quit  /: filter  Space: freeze    ↑↓: scroll  Enter: expand")
                }
            };

            // Find the expanded entry using scroll position in the filtered list
            let expanded_detail = if expanded {
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
                let constraints = if expanded {
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
                        .style(Style::default().fg(GRB_BG4).bg(GRB_BG0));
                    f.render_widget(empty, chunks[0]);
                } else {
                    let list = List::new(items)
                        .block(Block::default().borders(Borders::NONE)
                        .style(Style::default().bg(GRB_BG0)));
                    f.render_widget(list, chunks[0]);
                }

                if let Some(detail) = expanded_detail {
                    // Colorize JSON lines in the detail panel
                    let colored_lines: Vec<ratatui::text::Line> = detail
                    .lines()
                    .map(|l| {
                            let trimmed = l.trim();
                            if trimmed == "{" || trimmed == "}" || trimmed == "{," || trimmed == "}," {
                                ratatui::text::Line::from(
                                    ratatui::text::Span::styled(l.to_string(), Style::default().fg(GRB_BG4))
                                )
                            } else if trimmed.starts_with('"') && trimmed.contains(':') {
                                if let Some(colon) = trimmed.find(':') {
                                    let key = &trimmed[..colon + 1];
                                    let value = trimmed[colon + 1..].trim();
                                    let indent = &l[..l.len() - l.trim_start().len()];

                                    let value_style = if value.starts_with('"') {
                                        Style::default().fg(GRB_GREEN_L)
                                    } else if value == "true" || value == "false" {
                                        Style::default().fg(GRB_ORANGE_L)
                                    } else if value.starts_with(|c: char| c.is_numeric() || c == '-') {
                                        Style::default().fg(GRB_PURPLE_L)
                                    } else {
                                        Style::default().fg(GRB_FG2)
                                    };

                                    ratatui::text::Line::from(vec![
                                        ratatui::text::Span::raw(indent.to_string()),
                                        ratatui::text::Span::styled(key.to_string(), Style::default().fg(GRB_AQUA_L)),
                                        ratatui::text::Span::raw(" "),
                                        ratatui::text::Span::styled(value.to_string(), value_style),
                                    ])
                                } else {
                                    ratatui::text::Line::from(
                                        ratatui::text::Span::styled(l.to_string(), Style::default().fg(GRB_FG1))
                                    )
                                }
                            } else {
                                ratatui::text::Line::from(
                                    ratatui::text::Span::styled(l.to_string(), Style::default().fg(GRB_FG2))
                                )
                            }
                        })
                        .collect();

                        let panel = ratatui::widgets::Paragraph::new(colored_lines)
                            .block(Block::default()
                                .borders(Borders::ALL)
                                .title(" -> Detail ")
                                .border_style(Style::default().fg(GRB_BG3))
                                .title_style(Style::default().fg(GRB_YELLOW).add_modifier(Modifier::BOLD))
                                .style(Style::default().bg(GRB_BG0_H)))
                            .style(Style::default().bg(GRB_BG0_H));
                    f.render_widget(panel, chunks[1]);
                }
        
                let status_widget = ratatui::widgets::Paragraph::new(format!(" {}", status))
                    .style(Style::default().fg(GRB_BG0).bg(GRB_YELLOW));
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
                        if frozen {
                            let buf = buffer.read().unwrap();
                            frozen_entries = buf.filtered(&query, CONTEXT_LINES)
                                .into_iter()
                                .map(|(i, m, e)| (i, m, e.clone()))
                                .collect();
                            scroll = frozen_entries.len().saturating_sub(1);
                            view_offset = scroll.saturating_sub(
                                terminal.size()?.height as usize / 2
                            );
                        } else {
                            frozen_entries.clear();
                            expanded = false;
                        }
                    }
                    KeyCode::Char('/') if !typing => {  // "/" -> fileter bar
                        typing = true;
                    }
                    KeyCode::Esc => {                   // "Esc" -> exits typing mode
                        typing = false;                 //-And clears the search.
                        query.clear();
                    }
                    KeyCode::Enter => {
                        if typing {
                            // if in search bar, closes it.
                            typing = false
                        }else{
                            // if navigating toggles a detailed panel
                            frozen = true;
                            if frozen_entries.is_empty() {
                                let buf = buffer.read().unwrap();
                                frozen_entries = buf.filtered(&query, CONTEXT_LINES)
                                    .into_iter()
                                    .map(|(i, m, e)| (i, m, e.clone()))
                                    .collect();
                                scroll = frozen_entries.len().saturating_sub(1);
                                view_offset = scroll.saturating_sub(terminal.size()?.height as usize / 2);
                            }
                            if expanded {
                                expanded = false;
                            } else {
                                expanded = true;
                            }
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
                        if frozen_entries.is_empty() {
                            let buf = buffer.read().unwrap();
                            frozen_entries = buf.filtered(&query, CONTEXT_LINES)
                                .into_iter()
                                .map(|(i, m, e)| (i, m, e.clone()))
                                .collect();
                            scroll = frozen_entries.len().saturating_sub(1);
                        }
                        scroll = scroll.saturating_sub(1);
                    }
                    KeyCode::Down => {
                        frozen = true;
                        // Graphs the read lock on the ring buffer to check how many entries
                        //-currently exist.
                        if frozen_entries.is_empty() {
                            let buf = buffer.read().unwrap();
                            frozen_entries = buf.filtered(&query, CONTEXT_LINES)
                                .into_iter()
                                .map(|(i, m, e)| (i, m, e.clone()))
                                .collect();
                            scroll = frozen_entries.len().saturating_sub(1);
                        }
                        if scroll + 1 < frozen_entries.len() {
                            scroll += 1;
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
