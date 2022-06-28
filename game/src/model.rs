use crate::persistence::Persisted;

#[derive(Debug, Persisted)]
pub struct EntityKind {
    pub id: usize,
    pub name: String,
    pub triangle: usize,
    pub quad: usize,
}

#[derive(Debug, Persisted)]
pub struct Entity {
    pub id: usize,
    pub triangle: usize,
    pub quad: usize,
}
