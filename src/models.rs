use diesel::Queryable;

#[derive(Queryable)]
pub struct Tunnel {
    pub id: i32,
    pub domain_from: String,
    pub domain_to: String,
}