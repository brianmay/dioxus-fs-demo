use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use crate::server::schema::penguin_encounter;

#[cfg(feature = "server")]
use diesel::prelude::*;

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(diesel_derive_enum::DbEnum, Debug))]
#[cfg_attr(
    feature = "server",
    ExistingTypePath = "crate::schema::sql_types::PenaltyEnum"
)]
pub enum PenaltyEnum {
    PatPenguin,
    BecomePenguinGood,
    Jail,
    Sacrifice,
    WorshipTux,
}

impl std::fmt::Display for PenaltyEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PenaltyEnum::PatPenguin => write!(f, "Pat Penguin"),
            PenaltyEnum::BecomePenguinGood => write!(f, "Become Penguin Good"),
            PenaltyEnum::Jail => write!(f, "Jail"),
            PenaltyEnum::Sacrifice => write!(f, "Sacrifice"),
            PenaltyEnum::WorshipTux => write!(f, "Worship Tux"),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(Queryable, Selectable))]
#[cfg_attr(feature = "server", diesel(table_name = penguin_encounter))]
#[cfg_attr(feature = "server", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct PenguinEncounter {
    pub id: i32,
    pub name: String,
    pub location: String,
    pub penalty: PenaltyEnum,
    pub date_time: chrono::NaiveDateTime,
}

#[allow(dead_code)]
#[cfg_attr(feature = "server", derive(Insertable))]
#[cfg_attr(feature = "server", diesel(table_name = penguin_encounter))]
pub struct CreatePenguinEncounter<'a> {
    pub name: &'a str,
    pub location: &'a str,
    pub penalty: PenaltyEnum,
    pub date_time: chrono::NaiveDateTime,
}
