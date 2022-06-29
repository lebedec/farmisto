use rusqlite::{Row, Statement};

pub trait Persist: Sized {
    type Kind;

    fn entry_id(&self) -> usize {
        unimplemented!()
    }

    fn columns() -> Vec<String>;

    fn bind(&self, statement: &mut Statement) -> rusqlite::Result<()>;

    #[allow(unused_variables)]
    fn parse(row: &Row) -> Result<Self, rusqlite::Error> {
        unimplemented!()
    }

    #[allow(unused_variables)]
    fn parse_known(row: &Row, kind: Self::Kind) -> Result<Self, rusqlite::Error> {
        unimplemented!()
    }

    fn group() -> String {
        unimplemented!()
    }
}
