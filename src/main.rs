extern crate diesel;
extern crate ragger_api;
extern crate clap;

use self::diesel::prelude::*;
use self::ragger_api::models::*;
use self::ragger_api::*;
use clap::App;
use std::io::{stdin, stdout, Write};

// #[derive(StructOpt)]
// struct Cli{
//     operation: String,
//     word: String,
// }


fn main() {
    // let args = Cli::from_args();
    // println!("参数是: operation: {}, word: {}", &args.operation, &args.word);

    let matches = App::new("ragger")
        .version("1.0")
        .author("Lv Piao 1243194544@qq.com")
        .about("单词复习功能")
        .arg("<operation> 操作:query/review")
        .arg("[param] 操作参数:如果是query则为要查询的单词")
        .get_matches();
    if let Some(o) = matches.value_of("operation") {
        if o.eq("query") {
            if let Some(p) = matches.value_of("param") {
                let youdao_searcher = YoudaoSearcher::new(YoudaoConfig {
                    appkey: String::from("2f3a48a702316da0"),
                    appsecret: String::from("gTzzXsjEpFX9wUd8Rrio5SXfgzlcqq56"),
                });
                let values = youdao_searcher.search(String::from(p));
                //todo 将查询出来的单词插入数据库
                let conn = establish_connection();
                println!("单词的释义是: {:?}", &values);
                create_word(&conn, p, serde_json::to_string(&values).unwrap().as_str())
            } else {
                println!("请输入param参数");
            }
        } else if o.eq("review") {
            //整理命令
            //_exit 退出复习
            //_skip 跳过当前复习的单词
            //_remind
            let mut supervisor = Supervisor::initialize();
            if !supervisor.is_reviewable(){
                println!("当前没有可供复习的单词!");
                return;
            }
            println!("确认开始复习吗?(Y/N)");
            loop {
                let mut confirmation = String::new();
                stdin().read_line(&mut confirmation).unwrap();
                match confirmation.trim() {
                    "Y"=>{
                        println!("接下来将开始复习, 请按照控制台输出的单词输入其释义");
                        break;
                    },
                    "N"=>return,
                    _ =>println!("请输入'Y'来确认开始复习或者输入'N'退出复习")

                }
            }
            //todo 如果vec的长度为空则不用循环了
            // println!(">确认开始复习吗? Y/N");
            loop {
                if let Some(current_word) = supervisor.get_current_word(){
                    match stdout().flush() {
                        Err(_e) => panic!("刷新输出失败"),
                        _ => {}
                    }
                    println!("> {}", current_word.name);
                    let mut input = String::new();
                    stdin().read_line(&mut input).unwrap();
                    let answer = input.trim();
                    match answer {
                        //todo
                        // _prompt 给出例句提示
                        // _skip 跳过当前复习的单词, 不给答案, 不算次数
                        // _cheat 直接给出答案, 不算次数
                        "_exit" => break,//放弃复习
                        _ => {
                            if supervisor.exam(&answer) {
                                println!("回答正确");
                                supervisor.set_word_reviewed_once();
                            } else {
                                println!("回答内容是:{}", answer);
                                println!("回答错误，正确答案释义是: {}", current_word.exp_cn);
                            }
                            if supervisor.is_finished() {
                                //todo 复习完毕之后输出总结，包括成功次数，失败次数，共复习次数
                                println!("复习已完毕");
                                break;
                            } else {
                                supervisor.forward();
                            }
                        }
                    }
                    //todo 采集需要复习的单词并且显示出来
                    // let mut child = Command::new(command).spawn().unwrap();
                    // child.wait();
                } else {
                    panic!("程序出错")
                };
            }
        } else {
            println!("无法识别的operation参数");
        }
    }


    // let connection = establish_connection();
    // create_word(&connection, "hello", serde_json::to_string(&values).unwrap().as_str());
}


pub fn create_word<'a>(conn: &SqliteConnection, name_field: &'a str, exp_cn_field: &'a str) {
    use ragger_api::schema::words;
    use ragger_api::schema::words::dsl::*;
    let new_word = NewWord {
        name: name_field,
        exp_cn: exp_cn_field,
    };
    println!("要插入的数据是 {:?}", &new_word);
    diesel::insert_into(words::table).values(new_word).execute(conn).expect("插入数据失败");
    let word_results = words.filter(status.eq(0)).limit(10).load::<Word>(conn).expect("查询出错了");
    for word in word_results {
        println!("单词名称是:{}, 单词释义:{}", word.name, word.exp_cn);
    }
    // println!("查询到了{}条数据", word_results.len());
}





