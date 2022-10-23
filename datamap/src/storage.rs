use rusqlite::types::FromSql;
use rusqlite::Connection;
use std::path::Path;

pub struct Storage {
    connection: Connection,
    last_change_timestamp: usize,
}

impl Storage {
    pub fn open<P: AsRef<Path>>(path: P) -> rusqlite::Result<Self> {
        Connection::open(path).map(|connection| Storage {
            connection,
            last_change_timestamp: 0,
        })
    }

    #[inline]
    pub fn connection(&self) -> &Connection {
        &self.connection
    }

    pub fn track_changes<T>(&mut self) -> Result<Vec<Change<T>>, rusqlite::Error>
    where
        T: FromSql,
    {
        let mut changes = vec![];
        let mut statement = self
            .connection
            .prepare("select * from sql_tracking where timestamp > ?")?;
        let mut rows = statement.query([self.last_change_timestamp])?;
        while let Some(row) = rows.next()? {
            let timestamp: usize = row.get("timestamp")?;
            let entity: String = row.get("entity")?;
            let id = row.get("id")?;
            let operation: String = row.get("operation")?;
            let operation = match operation.as_str() {
                "Insert" => Operation::Insert,
                "Update" => Operation::Update,
                "Delete" => Operation::Delete,
                _ => return Err(rusqlite::Error::InvalidParameterName(operation)),
            };
            changes.push(Change {
                timestamp,
                entity,
                id,
                operation,
            });
            self.last_change_timestamp = timestamp;
        }
        Ok(changes)
    }

    pub fn setup_tracking(&self) -> Result<usize, rusqlite::Error> {
        let tracking_table = "create table if not exists sql_tracking (
            timestamp integer primary key autoincrement,
            entity text not null,
            id blob not null,
            operation text not null
        );";
        self.connection.execute(tracking_table, [])?;

        let insert = "create trigger if not exists on_<table>_insert
        after insert on <table>
        begin
            insert into sql_tracking (entity, id, operation)
            values ('<table>', new.id, 'Insert');
        end;";

        let update = "create trigger if not exists on_<table>_update
        after update on <table>
        begin
            insert into sql_tracking (entity, id, operation)
            values ('<table>', new.id, 'Update');
        end;";

        let delete = "create trigger if not exists on_<table>_delete
        after delete on <table>
        begin
            insert into sql_tracking (entity, id, operation)
            values ('<table>', old.id, 'Delete');
        end;";

        let mut statement = self
            .connection
            .prepare("select * from sqlite_master where type = 'table'")?;
        let mut rows = statement.query([])?;
        while let Some(row) = rows.next()? {
            let name: String = row.get("name")?;
            if name.starts_with("sql") {
                // skip sqlite tables
                continue;
            }
            self.connection
                .execute(&insert.replace("<table>", &name), [])?;
            self.connection
                .execute(&update.replace("<table>", &name), [])?;
            self.connection
                .execute(&delete.replace("<table>", &name), [])?;
        }

        self.connection.execute("delete from sql_tracking", [])
    }
}

#[derive(Debug)]
pub struct Change<T> {
    pub timestamp: usize,
    pub entity: String,
    pub id: T,
    pub operation: Operation,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operation {
    Insert,
    Update,
    Delete,
}
