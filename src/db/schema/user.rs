use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "user")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    pub chat_id: i64,
    pub child: Option<i64>,
    pub santa: Option<i64>,
    pub nickname: String,
    pub username: String,
    pub wish_text: String,
    pub state: String,
    pub city: String,
    pub create_date: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
