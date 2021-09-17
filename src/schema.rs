table! {
    jobs (id) {
        id -> Integer,
        repository -> Text,
        hash -> Text,
        comments_url -> Text,
        started_at -> Nullable<Timestamp>,
    }
}
