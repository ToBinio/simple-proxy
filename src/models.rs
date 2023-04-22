use diesel::Queryable;
use serde::Serialize;

#[derive(Queryable, Debug, Serialize)]
pub struct Tunnel {
    pub id: i32,
    pub domain_from: String,
    pub domain_to: String,
}
