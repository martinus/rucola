extern crate regex;
extern crate toml;

use std::io::{BufRead, BufReader, Write, Read};
use std::process::{Command, Stdio};
use std::thread;
use std::fs::File;
use std::env;
use regex::Regex;


fn parse(filename: &str) {
    let mut input = String::new();
    File::open(&filename).and_then(|mut f| {
            f.read_to_string(&mut input)
    }).unwrap();    

    let mut parser = toml::Parser::new(&input);
    let toml = match parser.parse() {
        Some(toml) => toml,
        None => {
            for err in &parser.errors {
                let (loline, locol) = parser.to_linecol(err.lo);
                let (hiline, hicol) = parser.to_linecol(err.hi);
                println!("{}:{}:{}-{}:{} error: {}",
                         filename, loline, locol, hiline, hicol, err.desc);
            }
            return
        }
    };
    println!("{:?}", toml);
}


fn handle<W: Write>(out: &mut W, re: &Regex, line: String) {
    if re.is_match(&line) {
        writeln!(out, "\x1B[33m{}\x1B[0m", line).unwrap();
    } else {
        writeln!(out, "{}", line).unwrap();
    }
}

/*
#[derive(RustcDecodable, RustcEncodable)]
pub struct Rules  {
    cmd: String,
    rules: Vec<Vec<String>>,
}
*/

fn main() {
    parse("test.toml");


    // Serialize using `json::encode`
    //let encoded = json::as_pretty_json(&o);
    //println!("{}", encoded);

    let re = Regex::new(r"h\wme").unwrap();

    let mut args: Vec<String> = env::args().collect();
    args.remove(0);
    let cmd: String = args.remove(0);

    let mut child = Command::new(cmd)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    // process stderr in other thread
    let mut child_stderr = child.stderr;
    let r = re.clone();
    let stderr_thread = thread::spawn(move || {
        if let Some(ref mut stderr) = child_stderr {
            let self_stderr = std::io::stderr();
            let mut h = self_stderr.lock();
            for line in BufReader::new(stderr).lines() {
                handle(&mut h, &r, line.unwrap());
            }
        }
        return child_stderr;
    });

    if let Some(ref mut stdout) = child.stdout {
        let self_stdout = std::io::stdout();
        let mut h = self_stdout.lock();
        for line in BufReader::new(stdout).lines() {
            handle(&mut h, &re, line.unwrap());
        }
    }

    // set back, so we can modify child again
    child.stderr = stderr_thread.join().unwrap();

    match child.wait() {
        Ok(c) => std::process::exit(c.code().unwrap()),
        Err(e) => println!("err={:?}", e),
    }
}
