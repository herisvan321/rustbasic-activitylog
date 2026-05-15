use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "activity_log")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub log_name: Option<String>,
    #[sea_orm(column_type = "Text")]
    pub description: String,
    pub subject_type: Option<String>,
    pub subject_id: Option<i32>,
    pub causer_type: Option<String>,
    pub causer_id: Option<i32>,
    pub properties: Option<Json>,
    pub created_at: Option<DateTime>,
    pub updated_at: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
