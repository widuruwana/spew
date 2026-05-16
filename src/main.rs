/* ---> Diary Entry #1 <---
  This project is my first time learning how to use Rust so you will
  see blocks of comments like these that either overly explain code
  or vent my personal frustrations, hopefull as less as possible.
*/

// ---> The Imports
// BufRead is what Rust calls a Trait (kind of like an interface/contract).
// BufRead unlocks the ability to read Streams line-by-line.
use std::io::{self, BufRead};
use serde_json::Value;

struct LogEntry {
    ts: String,
    level: String,
    msg: String,
    raw: String
}

fn parse_line(line: &str) -> LogEntry {
    // match is a superpowerd switch statemn -> rust wont compile unless code is
    // -written to handle every single possible outcome of it.
    // &line -> passing by reference. Allows the JSON parser to borrow the memory
    // address where the string lives, rather that copying the whole string to
    // the new buffer.
    // ::<serde_json::value> is called the "turbofish" syntax.
    // It tells the parser exactly what memory layout to use. In here we tells it to
    // parse the string to an unstrauctured, generic JSON tree (a Value).
    match serde_json::from_str::<Value>(line){
        Ok(json) => LogEntry {
            // Success branch -> Successfully parsed the JSON string
            // .unwrap_or() is an alternative to .unwrap()
            // in here instead of crashing it gives the fallback string "???"
            ts:     json["ts"].as_str().unwrap_or("???").to_string(),
            level:  json["level"].as_str().unwrap_or("???").to_string(),
            msg:    json["msg"].as_str().unwrap_or("???").to_string(),
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

fn main(){
    // io::stdin() grabs a global handle to the operating system's standard input stream
    // stdin is thread safe -> aquires and releases a mutex lock on every single byte it reads.
    /* io::stdin().read_line() will,
            1. Stops the thread.
            2. Reaches out to global stdin Mutex and lock the line.
            3. Reads that line.
            4. Unlcoks the mutex.
        Problem: Acquiring and releasing lets say 200k log lines per second will introduce
                overhead that will completely bottleneck the parse.
        Solution: io:stdin().lock() will grab the mutex and wont let go until the specific
                thread completely finished reading everything.
    */
    let stdin = io::stdin();
    for line in stdin.lock().lines(){
        
        // .lines() iterator returns Result<String, Error>
        // .unwrap() tells the compiler that it expect this to succeed but in a case
        // -of failing to read this memory address or stream, instantly crash(panic)
        // the program right here.
        let line = line.unwrap();
        let entry = parse_line(&line);

        // ! means println is a macro.
        // Rust expands the macro to ensure types and memory align perfectly
        // before the code even compiles
        if entry.ts.is_empty(){
            println!("{}", entry.raw);
        } else {
            println!("[{}] {:5} | {}", entry.ts, entry.level, entry.msg);
        }

    }
}
