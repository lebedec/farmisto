use log::info;
use rusqlite::types::{FromSql, ValueRef};
use rusqlite::{Connection, Statement};
use serde::de::DeserializeOwned;
use serde::de::Unexpected::Str;
use serde::Deserialize;
use serde_json::{Number, Value};
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;
use std::time::Instant;

pub struct Storage {
    connection: Connection,
    connection_string: String,
    last_change_timestamp: usize,
}

pub struct Entry {
    columns: Rc<HashMap<String, usize>>,
    values: Vec<Value>,
}

impl Entry {
    pub fn get<'a, T: Deserialize<'a>>(&'a self, index: &str) -> Result<T, serde_json::Error> {
        let index = *self.columns.get(index).unwrap();
        T::deserialize(&self.values[index])
        // serde_json::from_value(self.values[index].clone())
    }
}

impl Storage {
    pub fn open<P: AsRef<Path>>(path: P) -> rusqlite::Result<Self> {
        let t = Instant::now();
        let res = Connection::open(path.as_ref()).map(|connection| Storage {
            connection,
            connection_string: path.as_ref().to_str().unwrap().to_string(),
            last_change_timestamp: 0,
        });
        let t = t.elapsed();
        info!("CONNNNECT! {:?}", t);
        res
    }

    #[inline]
    pub fn connection(&self) -> &Connection {
        &self.connection
    }

    pub fn reopen(&self) -> Storage {
        Storage::open(&self.connection_string).unwrap()
    }

    pub fn relate2(&self, parent: &str) -> Result<Statement<'_>, rusqlite::Error> {
        let table = std::any::type_name::<Self>().split("::").last().unwrap();
        let mut statement = self
            .connection
            .prepare(&format!("select * from {} where {} = ?", table, parent))?;
        Ok(statement)
    }

    pub fn fetch<T>(&self, p0: &str, parent: &str) -> Entry {
        self.fetch_many::<T>(p0, parent).remove(0)
    }

    pub fn fetch_many<T>(&self, p0: &str, parent: &str) -> Vec<Entry> {
        let table = std::any::type_name::<T>().split("::").last().unwrap();
        let mut statement = self
            .connection
            .prepare(&format!("select * from {} where {} = ?", table, parent))
            .unwrap();
        let mut columns: HashMap<String, usize> = Default::default();
        for (index, column) in statement.column_names().iter().enumerate() {
            columns.insert(column.to_string(), index);
        }
        let columns_count = columns.len();
        let columns = Rc::new(columns);
        let mut rows = statement.query([p0]).unwrap();
        let mut entries = vec![];
        while let Some(row) = rows.next().unwrap() {
            let mut values = vec![];

            for i in 0..columns_count {
                let value = match row.get_ref_unwrap(i) {
                    ValueRef::Null => Value::Null,
                    ValueRef::Integer(data) => Value::Number(Number::from(data)),
                    ValueRef::Real(data) => Value::Number(Number::from_f64(data).unwrap()),
                    ValueRef::Text(ptr) => {
                        if ptr[0] == '[' as u8 || ptr[0] == '{' as u8 {
                            serde_json::from_slice(ptr).unwrap()
                        } else {
                            Value::String(String::from_utf8_lossy(ptr).to_string())
                        }
                    }
                    ValueRef::Blob(ptr) => serde_json::from_slice(ptr).unwrap(),
                };
                values.push(value);
            }
            let entry = Entry {
                columns: columns.clone(),
                values,
            };
            entries.push(entry);
        }
        entries
    }

    pub fn relate<T, M>(
        &self,
        parent: &str,
        parent_id: &str,
        mut map: M,
    ) -> Result<Vec<T>, rusqlite::Error>
    where
        M: FnMut(&rusqlite::Row) -> Result<T, rusqlite::Error>,
    {
        let table = std::any::type_name::<Self>().split("::").last().unwrap();
        let mut statement = self
            .connection
            .prepare(&format!("select * from {} where {} = ?", table, parent))?;
        let mut rows = statement.query([parent_id])?;
        let mut prefetch = vec![];
        while let Some(row) = rows.next()? {
            let value = map(row)?;
            prefetch.push(value);
        }
        Ok(prefetch)
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
