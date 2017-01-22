use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::thread;
use std::env;

fn handle<W: Write>(out: &mut W, line: String) {
    writeln!(out, "\x1B[33m{}\x1B[0m", line).unwrap();
}

fn main() {
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
    let stderr_thread = thread::spawn(move || {
        if let Some(ref mut stderr) = child_stderr {
            let self_stderr = std::io::stderr();
            let mut h = self_stderr.lock();
            for line in BufReader::new(stderr).lines() {
                handle(&mut h, line.unwrap());
            }
        }
        return child_stderr;
    });

    if let Some(ref mut stdout) = child.stdout {
        let self_stdout = std::io::stdout();
        let mut h = self_stdout.lock();
        for line in BufReader::new(stdout).lines() {
            handle(&mut h, line.unwrap());
        }
    }

    // set back, so we can modify child again
    child.stderr = stderr_thread.join().unwrap();

    match child.wait() {
        Ok(c) => std::process::exit(c.code().unwrap()),
        Err(e) => println!("err={:?}", e),
    }
}
