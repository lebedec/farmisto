use rusqlite::Connection;
use std::path::Path;

pub struct Storage {
    connection: Connection,
}

impl Storage {
    pub fn open<P: AsRef<Path>>(path: P) -> rusqlite::Result<Self> {
        Connection::open(path).map(|connection| Storage { connection })
    }

    #[inline]
    pub fn connection(&self) -> &Connection {
        &self.connection
    }
}
