use crate::engine::{MeshAsset, PropsAsset, TextureAsset};
use crate::Assets;
use datamap::WithContext;
use glam::Vec3;
use rusqlite::Connection;
use std::cell::RefCell;
use std::sync::Arc;

pub struct FarmlandAsset {
    pub data: Arc<RefCell<FarmlandAssetData>>,
}

pub struct FarmlandAssetData {
    pub props: Vec<FarmlandAssetPropItem>,
}

// TODO: autogenerate
impl WithContext for FarmlandAssetData {
    type Context = Assets;

    fn parse(
        row: &rusqlite::Row,
        id: usize,
        context: &mut Self::Context,
        connection: &rusqlite::Connection,
    ) -> Result<Self, rusqlite::Error> {
        Ok(Self {
            props: FarmlandAssetPropItem::prefetch(id, context, connection)?,
        })
    }
}

// TODO: autogenerate
impl From<Arc<RefCell<FarmlandAssetData>>> for FarmlandAsset {
    fn from(data: Arc<RefCell<FarmlandAssetData>>) -> Self {
        Self { data }
    }
}

pub struct FarmlandAssetPropItem {
    pub id: usize,
    pub farmland: usize,
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub scale: [f32; 3],
    pub asset: PropsAsset,
}

impl FarmlandAssetPropItem {
    #[inline]
    pub fn position(&self) -> Vec3 {
        Vec3::from(self.position)
    }
}

// TODO: autogenerate
impl WithContext for FarmlandAssetPropItem {
    type Context = Assets;

    fn prefetch(
        parent: usize,
        context: &mut Self::Context,
        connection: &Connection,
    ) -> Result<Vec<Self>, rusqlite::Error> {
        let table = std::any::type_name::<Self>().split("::").last().unwrap();
        let key = "farmland";
        let mut statement = connection
            .prepare(&format!("select * from {} where {} = ?", table, key))
            .unwrap();
        let mut rows = statement.query([parent]).unwrap();
        let mut prefetch = vec![];
        while let Some(row) = rows.next()? {
            let id: usize = row.get("id")?;
            let value = Self::parse(row, id, context, connection)?;
            prefetch.push(value);
        }
        Ok(prefetch)
    }

    fn parse(
        row: &rusqlite::Row,
        id: usize,
        context: &mut Self::Context,
        connection: &rusqlite::Connection,
    ) -> Result<Self, rusqlite::Error> {
        let asset: String = row.get("asset")?;

        Ok(Self {
            id,
            farmland: row.get("farmland")?,
            position: datamap::parse_json_value(row.get("position")?),
            rotation: datamap::parse_json_value(row.get("rotation")?),
            scale: datamap::parse_json_value(row.get("scale")?),
            asset: context.props(&asset),
        })
    }
}

#[derive(Clone)]
pub struct TreeAsset {
    data: Arc<RefCell<TreeAssetData>>,
}

pub struct TreeAssetData {
    pub texture: TextureAsset,
    pub mesh: MeshAsset,
}

impl TreeAsset {
    #[inline]
    pub fn texture(&self) -> TextureAsset {
        self.data.borrow().texture.clone()
    }

    #[inline]
    pub fn mesh(&self) -> MeshAsset {
        self.data.borrow().mesh.clone()
    }
}

// TODO: autogenerate
impl WithContext for TreeAssetData {
    type Context = Assets;

    fn parse(
        row: &rusqlite::Row,
        id: usize,
        context: &mut Self::Context,
        connection: &rusqlite::Connection,
    ) -> Result<Self, rusqlite::Error> {
        let texture: String = row.get("texture")?;
        let mesh: String = row.get("mesh")?;
        Ok(Self {
            texture: context.texture(texture),
            mesh: context.mesh(mesh),
        })
    }
}

// TODO: autogenerate
impl From<Arc<RefCell<TreeAssetData>>> for TreeAsset {
    fn from(data: Arc<RefCell<TreeAssetData>>) -> Self {
        Self { data }
    }
}
