use structopt::StructOpt;
use ragger_api::{YoudaoSearcher, YoudaoConfig, Searchable};

use ragger_api::models::*;
use ragger_api::establish_connection;
use diesel::prelude::*;

#[derive(StructOpt)]
struct Cli{
    operation: String,
    word: String,
}



fn main() {
    // let args = Cli::from_args();
    // println!("参数是: operation: {}, word: {}", &args.operation, &args.word);

    // let json_string = "[1, 2, 3]";
    // let v :Vec<i32> = serde_json::from_str(json_string).unwrap();
    // let json_string = "{\"jack\":\"u\"}";
    /*
    let json_string = "{\"jack\":\"u\",\"fuck\":\"u\", \"code\": 1}";
    let demo:Demo = serde_json::from_str(json_string).unwrap();
    println!("解码后的数据是 {}", demo.jack);
     */

    let youdao_searcher = YoudaoSearcher::new(YoudaoConfig{
        appkey: String::from("2f3a48a702316da0"),
        appsecret: String::from("gTzzXsjEpFX9wUd8Rrio5SXfgzlcqq56"),
    });
    let values = youdao_searcher.search(String::from("hello"));
    println!("执行结果是: {:?}", values);

    let connection = establish_connection();
    create_word(&connection, "hello", serde_json::to_string(&values).unwrap().as_str());





}

// pub fn establish_connection() -> SqliteConnection{
//     dotenv().ok();
//     let database_url = env::var("DATABASE_URL").expect("DATABASE_URL未设置");
//     SqliteConnection::establish(&database_url).expect(&format!("数据库连接出错, 请检查url配置"))
//
// }

pub fn create_word<'a>(conn: &SqliteConnection, name: &'a str, exp_cn: &'a str){
    use ragger_api::schema::words::dsl::*;
    // let new_word = NewWord{
    //     name,
    //     exp_cn
    // };

    let results = words.filter(name.eq("hello")).load::<Word>(&conn).expect("查询出错");
    println!("查询出了 {} 条数据", results.len());
    // println!("要插入的数据是 {:?}", &new_word);
    // diesel::insert_into(words::table).values(new_word).execute(conn).expect("插入数据失败");
}




