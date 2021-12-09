use super::schema::words;

#[derive(Queryable, Clone)]
pub struct Word {
    pub id: i32,
    pub name: String,
    pub exp_cn: String,
    pub status: i32,
}


#[derive(Insertable)]
#[table_name = "words"]
#[derive(Debug)]
pub struct NewWord<'a> {
    pub name: &'a str,
    pub exp_cn: &'a str,
}


