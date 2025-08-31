// @generated automatically by Diesel CLI.

diesel::table! {
    activities (id) {
        id -> Integer,
        date -> Date,
        activity_type -> Text,
        description -> Text,
    }
}

diesel::table! {
    balances (id) {
        id -> Integer,
        method_id -> Integer,
        year -> Integer,
        month -> Integer,
        balance -> BigInt,
        is_final_balance -> Bool,
    }
}

diesel::table! {
    tags (id) {
        id -> Integer,
        name -> Text,
    }
}

diesel::table! {
    tx_methods (id) {
        id -> Integer,
        name -> Text,
        position -> Integer,
    }
}

diesel::table! {
    tx_tags (tx_id, tag_id) {
        tx_id -> Integer,
        tag_id -> Integer,
    }
}

diesel::table! {
    txs (id) {
        id -> Integer,
        date -> Date,
        details -> Nullable<Text>,
        from_method -> Integer,
        to_method -> Nullable<Integer>,
        amount -> BigInt,
        tx_type -> Text,
        activity_id -> Nullable<Integer>,
        display_order -> Integer,
    }
}

diesel::joinable!(balances -> tx_methods (method_id));
diesel::joinable!(tx_tags -> tags (tag_id));
diesel::joinable!(tx_tags -> txs (tx_id));
diesel::joinable!(txs -> activities (activity_id));

diesel::allow_tables_to_appear_in_same_query!(activities, balances, tags, tx_methods, tx_tags, txs,);
