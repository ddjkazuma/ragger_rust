-- Your SQL goes here
create table words(
    id integer primary key autoincrement not null,
    name varchar(255) not null default '',
    exp_cn varchar(255) not null default '',
    status integer not null default 0
)