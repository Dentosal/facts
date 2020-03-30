use crossbeam_channel::{bounded, select, Receiver, Sender};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashSet;
use std::io::{BufRead, BufReader};
use std::process::{Child, ChildStdout};
use std::str::FromStr;
use std::thread::{self, JoinHandle};
use strum_macros::EnumString;

use crate::error::ServerError;

#[derive(Debug, Clone, Copy, EnumString, PartialEq, Eq)]
pub enum RunningServerState {
    Start,
    Ready,
    PreparedToHostGame,
    CreatingGame,
    InitializationFailed,
    InGame,
    InGameSavingMap,
    DisconnectingScheduled,
    Disconnecting,
    Disconnected,
    Closed,
    Failed,
}
impl Default for RunningServerState {
    fn default() -> Self {
        Self::Start
    }
}

#[derive(Debug, Clone, Default)]
pub struct RunningServer {
    pub state: RunningServerState,
    pub players_online: HashSet<String>,
}
impl RunningServer {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn new_line(&mut self, line: &str) -> Result<(), ServerError> {
        lazy_static! {
            static ref RE_ERROR: Regex =
                Regex::new(r"^\s*\d+\.\d+\sError\s.+?\s+(?P<msg>.+?)\s*$").unwrap();
            static ref RE_STATE: Regex = Regex::new(
                r"^\s*\d+\.\d+\sInfo.+?changing\sstate\sfrom\(.+?\)\sto\((?P<newstate>.+?)\)$"
            )
            .unwrap();
            static ref RE_EVENT: Regex = Regex::new(
                r"^\d{4,}+-\d{2}-\d{2}\s\d{2}:\d{2}:\d{2}\s\[(?P<event>.+?)\]\s(?P<msg>.+)$"
            )
            .unwrap();
        }

        if let Some(cap) = RE_ERROR.captures(line) {
            return if &cap["msg"] == "MultiplayerManager failed: Host address is already in use." {
                Err(ServerError::PortUnavailable)
            } else {
                panic!("Unknown Factorio server error: {:?}", &cap["msg"])
            };
        } else if let Some(cap) = RE_STATE.captures(line) {
            self.state = RunningServerState::from_str(&cap["newstate"])
                .unwrap_or_else(|_| panic!("Unknown state {}", &cap["newstate"]));
        } else if let Some(cap) = RE_EVENT.captures(line) {
            log::info!("[{}] {}", &cap["event"], &cap["msg"]);

            if &cap["event"] == "JOIN" {
                let name = cap["msg"].split_whitespace().next().unwrap();
                self.players_online.insert(name.to_owned());
            } else if &cap["event"] == "LEAVE" {
                let name = cap["msg"].split_whitespace().next().unwrap();
                self.players_online.remove(name);
                log::info!("{} left the game", name);
            }
        }

        Ok(())
    }
}

pub mod message {
    use super::RunningServer;

    #[derive(Debug, Clone)]
    pub enum ToServer {
        Shutdown,
        GetState,
    }

    #[derive(Debug, Clone)]
    pub enum FromServer {
        StartupComplete,
        State(RunningServer),
    }
}

/// Ignores send errors, as if the parent has crashed they don't matter anymore
fn stdout_thread(stdout: ChildStdout, tx: Sender<Option<String>>) {
    lazy_static! {
        static ref RE_GOODBYE: Regex = Regex::new(r"^\s*\d+\.\d+\sGoodbye$").unwrap();
    }

    let reader = BufReader::new(stdout);
    for line in reader.lines().filter_map(|line| line.ok()) {
        let done = RE_GOODBYE.is_match(&line);
        log::trace!("Factorio: {}", line);

        let _ = tx.send(Some(line));

        if done {
            break;
        }
    }

    let _ = tx.send(None);
}

pub fn run(
    mut child: Child, tx: Sender<message::FromServer>, rx: Receiver<message::ToServer>,
) -> Result<(), ServerError> {
    let (tx_stdout, rx_stdout) = bounded::<Option<String>>(0);
    let stdout = child.stdout.take().unwrap();
    let stdout_handle: JoinHandle<()> = thread::spawn(move || stdout_thread(stdout, tx_stdout));

    let mut state = RunningServer::new();
    let mut startup_complete = false;

    loop {
        select! {
            recv(rx) -> msg => match msg.expect("Recv from parent") {
                message::ToServer::Shutdown => {
                    // Send SIGINT, so that Factorio autosaves and quits
                    nix::sys::signal::kill(
                        nix::unistd::Pid::from_raw(child.id() as i32),
                        nix::sys::signal::Signal::SIGINT
                    ).expect("Could not send SIGINT");
                    break;
                },
                message::ToServer::GetState => {
                    tx.send(message::FromServer::State(state.clone())).unwrap();
                },
            },
            recv(rx_stdout) -> msg => match msg.expect("Recv from stdout") {
                Some(line) => state.new_line(&line)?,
                None => break,
            },
        };

        if !startup_complete && state.state == RunningServerState::InGame {
            startup_complete = true;
            tx.send(message::FromServer::StartupComplete).unwrap();
        }

        if state.state == RunningServerState::Closed {
            break;
        }
    }

    // Consume rest of the stdout
    while let Ok(Some(_)) = rx_stdout.recv() {}
    stdout_handle.join().expect("Stdout process crashed");

    let exitstatus = child.wait().expect("Server process did not start at all");
    assert!(exitstatus.success(), "Server process exited unsuccesfully");

    Ok(())
}
