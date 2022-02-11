table! {
    job (id) {
        id -> Integer,
        repository -> Text,
        hash -> Nullable<Text>,
        comments_url -> Nullable<Text>,
        started_at -> Nullable<Timestamp>,
    }
}

table! {
    master_build (id) {
        id -> Integer,
        hash -> Text,
    }
}

allow_tables_to_appear_in_same_query!(job, master_build,);
