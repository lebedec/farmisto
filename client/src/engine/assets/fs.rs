use log::{debug, error, info};
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, RwLock};
use std::time::Instant;
use std::{fs, thread};

pub struct FileSystem {
    events_timer: Instant,
    events: Arc<RwLock<HashMap<PathBuf, FileEvent>>>,
}

impl FileSystem {
    pub fn observe_file_events(&mut self) -> Vec<(PathBuf, FileEvent)> {
        if self.events_timer.elapsed().as_millis() >= 10 {
            self.events_timer = Instant::now();
            let mut events = match self.events.write() {
                Ok(events) => events,
                Err(error) => {
                    error!("Unable to observe file events, {:?}", error);
                    return vec![];
                }
            };
            std::mem::replace(&mut *events, HashMap::new())
                .into_iter()
                .collect()
        } else {
            // debounce events in some time
            vec![]
        }
    }

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
            compact_events(thread_events.clone(), path, event);
        });
        shared_events
    }

    pub fn idle() -> FileSystem {
        FileSystem {
            events_timer: Instant::now(),
            events: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    #[cfg(windows)]
    pub fn watch(extensions: Vec<&'static str>) -> FileSystem {
        let process = Command::new("powershell")
            .arg(include_str!("./includes/watcher.ps1"))
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to spawn powershell file watcher");

        info!("Starts file system watcher pid={}", process.id());

        let mut reader = BufReader::new(process.stdout.unwrap());
        let shared_events = Arc::new(RwLock::new(HashMap::new()));
        let thread_events = shared_events.clone();
        thread::Builder::new()
            .name("watch".into())
            .spawn(move || loop {
                let mut line = String::new();
                if let Err(error) = reader.read_line(&mut line) {
                    error!("watcher.ps1 finished {:?}", error);
                    break;
                }
                let line = line.trim();
                let ext = line.split(".").last().unwrap_or("");
                if !extensions.contains(&ext) {
                    continue;
                }
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
                compact_events(thread_events.clone(), path, event);
            })
            .unwrap();

        FileSystem {
            events_timer: Instant::now(),
            events: shared_events,
        }
    }
}

#[inline]
fn compact_events(
    events: Arc<RwLock<HashMap<PathBuf, FileEvent>>>,
    path: PathBuf,
    event: FileEvent,
) {
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
