// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "penalty_enum"))]
    pub struct PenaltyEnum;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::PenaltyEnum;

    penguin_encounter (id) {
        id -> Int4,
        name -> Varchar,
        location -> Varchar,
        penalty -> PenaltyEnum,
        date_time -> Timestamptz,
    }
}
