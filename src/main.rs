use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::thread;
use std::env;

fn handle(is_stdout: bool, line: String) {
    if is_stdout {
        println!("stdout: [{}]", line);
    } else {
        writeln!(&mut std::io::stderr(), "stderr [{}]", line);
    }
}

fn main() {
    let mut args: Vec<String> = env::args().collect();
    let self_cmd: String = args.remove(0);
    let cmd: String = args.remove(0);

    println!("{}", cmd);
    let mut child = Command::new(cmd)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    // process stderr in other thread
    let mut s = child.stderr;
    let stderr_thread = thread::spawn(move || {
        if let Some(ref mut stderr) = s {
            for line in BufReader::new(stderr).lines() {
                handle(false, line.unwrap());
            }
        }
    });

    if let Some(ref mut stdout) = child.stdout {
        for line in BufReader::new(stdout).lines() {
            handle(true, line.unwrap());
        }
    }

    stderr_thread.join();
}
