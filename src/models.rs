use diesel::Queryable;

#[derive(Queryable)]
pub struct Connection {
    pub id: i32,
    pub domain_from: String,
    pub domain_to: String,
}