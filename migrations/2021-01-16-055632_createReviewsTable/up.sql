-- Your SQL goes here
create table reviews(
    id integer primary key  autoincrement not null,
    word_id integer not null default '',
    date timestamp null,
    status integer not null default 0
)