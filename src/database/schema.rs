table! {
    addresses (ip, platform_id) {
        ip -> Inet,
        platform_id -> Int2,
    }
}

table! {
    results (id) {
        id -> Int8,
        platform_id -> Int2,
        generated_at -> Date,
        part_total -> Int8,
        part_valid -> Int8,
    }
}

table! {
    resultsgroupes (result_id, group_id) {
        result_id -> Int8,
        group_id -> Int2,
        value_median -> Float4,
        value_average -> Float4,
        value_uninominal -> Float4,
    }
}

table! {
    submissions (id) {
        id -> Int8,
        platform_id -> Int2,
        asn -> Int4,
        abuse_code -> Int2,
        sent_timestamp -> Date,
        duration -> Int4,
    }
}

table! {
    submissionschoices (submission_id, question_id) {
        submission_id -> Int8,
        question_id -> Int2,
        userchoice -> Int2,
    }
}

joinable!(resultsgroupes -> results (result_id));
joinable!(submissionschoices -> submissions (submission_id));

allow_tables_to_appear_in_same_query!(
    addresses,
    results,
    resultsgroupes,
    submissions,
    submissionschoices,
);
