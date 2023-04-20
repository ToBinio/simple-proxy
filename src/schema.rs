diesel::table! {
    connections (id) {
        id -> Int4,
        domain_from -> VarChar,
        domain_to -> VarChar,
    }
}