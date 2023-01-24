use rusqlite::types::{FromSql, ValueRef};
use rusqlite::{params, Connection, Params, Row};

use log::info;
use serde::Deserialize;
use serde_json::{Number, Value};
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

pub struct Storage {
    connection: Connection,
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
    }

    pub fn get_string(&self, index: &str) -> Result<&str, serde_json::Error> {
        self.get(index)
    }

    pub fn get_bool(&self, index: &str) -> Result<bool, serde_json::Error> {
        let value: i32 = self.get(index)?;
        Ok(value == 1)
    }
}

impl Storage {
    pub fn open<P: AsRef<Path>>(path: P) -> rusqlite::Result<Self> {
        Connection::open(path.as_ref()).map(|connection| Storage {
            connection,
            last_change_timestamp: 0,
        })
    }

    pub fn open_into(&self) -> Self {
        Connection::open(self.connection.path().unwrap())
            .map(|connection| Storage {
                connection,
                last_change_timestamp: 0,
            })
            .unwrap()
    }

    #[inline]
    pub fn connection(&self) -> &Connection {
        &self.connection
    }

    pub fn fetch_one<T>(&self, id: &str) -> Entry {
        self.query::<T, _>([id], "where id = ?").remove(0)
    }

    pub fn fetch_many<T>(&self, id: &str) -> Vec<Entry> {
        self.query::<T, _>([id], "where id = ?")
    }

    pub fn fetch_all<T>(&self) -> Vec<Entry> {
        self.query::<T, _>([], "")
    }

    fn query<T, P: Params>(&self, params: P, where_clause: &str) -> Vec<Entry> {
        let table = std::any::type_name::<T>().split("::").last().unwrap();
        let mut statement = self
            .connection
            .prepare(&format!("select * from {} {}", table, where_clause))
            .unwrap();
        let mut columns: HashMap<String, usize> = Default::default();
        for (index, column) in statement.column_names().iter().enumerate() {
            columns.insert(column.to_string(), index);
        }
        let columns_count = columns.len();
        let columns = Rc::new(columns);
        let mut rows = statement.query(params).unwrap();
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

    pub fn find_all<T, M>(&self, map: M) -> Vec<T>
    where
        M: FnMut(&Row) -> T,
    {
        self.query_map::<T, _, M>([], "", map)
    }

    pub fn fetch_one_map<T, M>(&self, id: &str, map: M) -> T
    where
        M: FnMut(&Row) -> T,
    {
        self.query_map::<T, _, M>([id], "where id = ?", map)
            .remove(0)
    }

    fn query_map<T, P: Params, M>(&self, params: P, where_clause: &str, mut map: M) -> Vec<T>
    where
        M: FnMut(&Row) -> T,
    {
        let table = std::any::type_name::<T>().split("::").last().unwrap();
        let mut statement = self
            .connection
            .prepare(&format!("select * from {} {}", table, where_clause))
            .unwrap();
        let mut rows = statement.query(params).unwrap();
        let mut values = vec![];
        while let Some(row) = rows.next().unwrap() {
            values.push(map(row));
        }
        values
    }

    pub fn select_changes<T>(
        &mut self,
        last_change_timestamp: usize,
        entity: &str,
    ) -> Result<Vec<Change<T>>, rusqlite::Error>
    where
        T: FromSql,
    {
        let mut changes = vec![];
        let mut statement = self
            .connection
            .prepare("select * from sql_tracking where timestamp > ? and entity = ?")?;
        let mut rows = statement.query(params![last_change_timestamp, entity])?;
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
        }
        Ok(changes)
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

    pub fn setup_tracking(&self) -> Result<(), rusqlite::Error> {
        let tracking_table = "-- drop table if exists sql_tracking;
        create table if not exists sql_tracking (
            timestamp integer primary key autoincrement,
            entity text not null,
            id blob not null,
            operation text not null
        );
        delete from sql_tracking where id is not null;";
        self.connection.execute_batch(tracking_table)?;

        let insert = "-- drop trigger if exists on_<table>_insert;
        create trigger if not exists on_<table>_insert
        after insert on <table>
        begin
            insert into sql_tracking (entity, id, operation)
            values ('<table>', new.id, 'Insert');
        end;";

        let update = "-- drop trigger if exists on_<table>_update;
        create trigger if not exists on_<table>_update
        after update on <table>
        begin
            insert into sql_tracking (entity, id, operation)
            values ('<table>', new.id, 'Update');
        end;";

        let delete = "-- drop trigger if exists on_<table>_delete;
        create trigger if not exists on_<table>_delete
        after delete on <table>
        begin
            insert into sql_tracking (entity, id, operation)
            values ('<table>', old.id, 'Delete');
        end;";

        let mut statement = self
            .connection
            .prepare("select * from sqlite_master where type = 'table'")?;

        let mut _foreign_keys = self
            .connection
            .prepare("select * from pragma_foreign_key_list(?);")?;

        //let mut references = HashMap::new();
        let mut tables = Vec::new();

        let mut rows = statement.query([])?;
        while let Some(row) = rows.next()? {
            let name: String = row.get("name")?;
            if name.starts_with("sql") {
                // skip sqlite tables
                continue;
            }
            tables.push(name);
        }

        for name in tables {
            info!("Initialize changes tracking for {}", name);
            self.connection
                .execute_batch(&insert.replace("<table>", &name))?;
            self.connection
                .execute_batch(&update.replace("<table>", &name))?;
            self.connection
                .execute_batch(&delete.replace("<table>", &name))?;
        }

        Ok(())
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
