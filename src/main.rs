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

fn main(){
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
        
    // temp print loop
    loop{
        // Pauses the main thread for 100 ms so its doesnt burn 100% of the CPU
        //-constantly checking for updates
        thread::sleep(std::time::Duration::from_millis(100));
        let buf = buffer.read().unwrap();
        let entries = buf.filtered("");
        for entry in entries {
            if entry.ts.is_empty(){
                println!("{}", entry.raw);
            } else {
                println!("[{}] {:5} | {}", entry.ts, entry.level, entry.msg);
            }
        }
        if !buf.entries.is_empty() {
            break;
        }
    }
}
