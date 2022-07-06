use log::{error, info};
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, RwLock};
use std::{fs, thread};

pub struct FileSystem {}

impl FileSystem {
    #[cfg(unix)]
    pub fn watch() -> Arc<RwLock<HashMap<PathBuf, FileEvent>>> {
        let process = Command::new("fswatch")
            .arg("assets")
            .arg("-xr")
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to spawn fswatch");
        let mut reader = BufReader::new(process.stdout.unwrap());
        let shared_events = Arc::new(RwLock::new(HashMap::new()));
        let thread_events = shared_events.clone();
        thread::spawn(move || loop {
            let mut line = String::new();
            if let Err(error) = reader.read_line(&mut line) {
                error!("fswatch finished {:?}", error);
                break;
            }
            let line = line.trim();
            let message = line.split(" ").collect::<Vec<&str>>();
            let (event, path) = match message[..] {
                [path, "Created", ..] => (FileEvent::Created, path),
                [path, .., "Updated"] => (FileEvent::Changed, path),
                [path, .., "Removed"] => (FileEvent::Deleted, path),
                _ => {
                    error!("invalid watcher event format, {}", line);
                    continue;
                }
            };
            let path = PathBuf::from(path.trim());

            let mut events = thread_events.write().unwrap();
            match events.get_mut(&path) {
                Some(entry) => {
                    *entry = match (&entry, event) {
                        (FileEvent::Created, FileEvent::Created) => FileEvent::Created,
                        (FileEvent::Created, FileEvent::Changed) => FileEvent::Changed,
                        (FileEvent::Created, FileEvent::Deleted) => FileEvent::Deleted,
                        (FileEvent::Changed, FileEvent::Created) => FileEvent::Changed,
                        (FileEvent::Changed, FileEvent::Changed) => FileEvent::Changed,
                        (FileEvent::Changed, FileEvent::Deleted) => FileEvent::Deleted,
                        (FileEvent::Deleted, FileEvent::Created) => FileEvent::Changed,
                        (FileEvent::Deleted, FileEvent::Changed) => FileEvent::Changed,
                        (FileEvent::Deleted, FileEvent::Deleted) => FileEvent::Deleted,
                    };
                }
                None => {
                    events.insert(path, event);
                }
            }
        });
        shared_events
    }

    #[cfg(windows)]
    pub fn watch() -> Arc<RwLock<HashMap<PathBuf, FileEvent>>> {
        let process = Command::new("powershell")
            .arg(include_str!("./includes/watcher.ps1"))
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to spawn powershell file watcher");

        let mut reader = BufReader::new(process.stdout.unwrap());
        let shared_events = Arc::new(RwLock::new(HashMap::new()));
        let thread_events = shared_events.clone();
        thread::spawn(move || loop {
            let mut line = String::new();
            if let Err(error) = reader.read_line(&mut line) {
                error!("fswatch finished {:?}", error);
                break;
            }
            let line = line.trim();
            let message = line.split(" ").collect::<Vec<&str>>();
            let (event, path) = match line.split(":").collect::<Vec<&str>>()[..] {
                ["Created", path] => (FileEvent::Created, path),
                ["Changed", path] => (FileEvent::Changed, path),
                ["Deleted", path] => (FileEvent::Deleted, path),
                _ => {
                    error!("invalid watcher event format, {}", line);
                    continue;
                }
            };
            let path = fs::canonicalize(".").unwrap().join(path.trim());
            info!("Try!!! [{:?}]", path.to_str());

            debounce_event(thread_events.clone(), path, event);
        });
        shared_events
    }
}

#[inline(always)]
fn debounce_event(events: Arc<RwLock<HashMap<PathBuf, FileEvent>>>, path: PathBuf, event: FileEvent) {
    let mut events = events.write().unwrap();
    match events.get_mut(&path) {
        Some(entry) => {
            *entry = match (&entry, event) {
                (FileEvent::Created, FileEvent::Created) => FileEvent::Created,
                (FileEvent::Created, FileEvent::Changed) => FileEvent::Changed,
                (FileEvent::Created, FileEvent::Deleted) => FileEvent::Deleted,
                (FileEvent::Changed, FileEvent::Created) => FileEvent::Changed,
                (FileEvent::Changed, FileEvent::Changed) => FileEvent::Changed,
                (FileEvent::Changed, FileEvent::Deleted) => FileEvent::Deleted,
                (FileEvent::Deleted, FileEvent::Created) => FileEvent::Changed,
                (FileEvent::Deleted, FileEvent::Changed) => FileEvent::Changed,
                (FileEvent::Deleted, FileEvent::Deleted) => FileEvent::Deleted,
            };
        }
        None => {
            events.insert(path, event);
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum FileEvent {
    Created,
    Changed,
    Deleted,
}
