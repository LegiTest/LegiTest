CREATE TABLE IF NOT EXISTS Addresses (
    ip inet NOT NULL,
    platform_id smallint NOT NULL,
    PRIMARY KEY (ip, platform_id)
);

CREATE TABLE IF NOT EXISTS Submissions (
    id bigserial NOT NULL PRIMARY KEY,
    platform_id smallint NOT NULL,
    asn integer NOT NULL,
    abuse_code smallint NOT NULL,
    sent_timestamp date NOT NULL,
    duration integer NOT NULL
);

CREATE TABLE IF NOT EXISTS SubmissionsChoices (
    submission_id bigint NOT NULL REFERENCES Submissions(id),
    question_id smallint NOT NULL,
    userchoice smallint NOT NULL,
    PRIMARY KEY (submission_id, question_id)
);

CREATE TABLE IF NOT EXISTS Results (
    id bigserial NOT NULL PRIMARY KEY,
    platform_id smallint NOT NULL,
    generated_at date NOT NULL,
    part_total bigint NOT NULL,
    part_valid bigint NOT NULL
);

CREATE TABLE IF NOT EXISTS ResultsGroupes (
    result_id bigint NOT NULL REFERENCES Results(id),
    group_id smallint NOT NULL,
    value_median real NOT NULL,
    PRIMARY KEY (result_id, group_id)
);
