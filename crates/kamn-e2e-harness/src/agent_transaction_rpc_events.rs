use std::io::{BufRead, BufReader};
use std::process::ChildStdout;
use std::sync::mpsc::{self, Receiver};
use std::thread::JoinHandle;

use serde_json::Value;

pub(super) fn spawn_reader(
    stdout: ChildStdout,
) -> (Receiver<Result<Value, String>>, JoinHandle<()>) {
    let (sender, receiver) = mpsc::channel();
    let handle = std::thread::spawn(move || read_events(stdout, sender));
    (receiver, handle)
}

fn read_events(stdout: ChildStdout, sender: mpsc::Sender<Result<Value, String>>) {
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) | Err(_) => break,
            Ok(_) if send_event(line.as_str(), &sender).is_err() => break,
            Ok(_) => {}
        }
    }
}

fn send_event(line: &str, sender: &mpsc::Sender<Result<Value, String>>) -> Result<(), ()> {
    let event = serde_json::from_str(line.trim())
        .map_err(|_| "AGENT_TRANSACTION_CHILD_FAILED: Pi RPC emitted malformed JSON".to_owned());
    sender.send(event).map_err(|_| ())
}
