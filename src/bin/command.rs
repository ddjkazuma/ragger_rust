extern crate clap;
extern crate diesel;
extern crate ragger_api;

use clap::{App, Arg, arg};
use diesel::prelude::*;
use ragger_api::models::*;
use ragger_api::schema::words;
use ragger_api::schema::words::dsl::*;
use ragger_api::*;
use std::io::{stdin, stdout, Write};
use termion::color;

// #[derive(StructOpt)]
// struct Cli{
//     operation: String,
//     word: String,
// }

fn main() {
    // let args = Cli::from_args();
    // println!("参数是: operation: {}, word: {}", &args.operation, &args.word);
    // println!("{}Red", color::Fg(color::Red));
    // return;

    let matches = App::new("ragger")
        .version("1.0")
        .author("Lv Piao 1243194544@qq.com")
        .about("单词复习功能")
        .arg(
            arg!(<operation> "操作:query/review/list/remove")
        )
        .arg(
            arg!([param] "操作参数:如果是query则为要查询的单词")
        )
        .arg(
            Arg::new("limit")
                .short('l')
                .long("limit")
                .takes_value(true)
                .help("设置本次复习的最大单词数量"),
        )
        .get_matches();
    if let Some(o) = matches.value_of("operation") {
        if o.eq("query") {
            if let Some(p) = matches.value_of("param") {
                //todo 先查询一下这个单词在数据库里是否存在
                let conn = establish_connection();
                let word_results: Vec<Word> = words
                    .filter(name.eq(p))
                    .limit(1)
                    .load::<Word>(&conn)
                    .expect("查询出错了");
                if word_results.len() > 0 {
                    println!(
                        "{}单词的释义是: {}",
                        color::Fg(color::Green),
                        word_results.get(0).unwrap().exp_cn
                    );
                } else {
                    let youdao_searcher = YoudaoSearcher::new(YoudaoConfig {
                        appkey: String::from("2f3a48a702316da0"),
                        appsecret: String::from("gTzzXsjEpFX9wUd8Rrio5SXfgzlcqq56"),
                    });
                    match youdao_searcher.search(String::from(p)) {
                        Ok(values) => {
                            println!("{}单词的释义是: {:?}", color::Fg(color::Green), &values);
                            create_word(&conn, p, serde_json::to_string(&values).unwrap().as_str());
                        }
                        Err(_) => {
                            // println!("{}有道api查询错误, 错误原因; {:?}", color::Fg(color::Red), error);
                            println!(
                                "{}无法查找到该单词的释义，请确认该单词拼写是否正确",
                                color::Fg(color::Red)
                            );
                        }
                    }
                }
            } else {
                println!("{}请输入param参数", color::Fg(color::Red));
            }
        } else if o.eq("review") {
            //整理命令
            //_exit 退出复习
            //_skip 跳过当前复习的单词
            //_remind
            let mut supervisor = match matches.value_of("limit") {
                Some(limit) => Supervisor::initialize(Some(limit.parse().unwrap())),
                None => Supervisor::initialize(None),
            };
            if !supervisor.is_reviewable() {
                println!("{}当前没有可供复习的单词!", color::Fg(color::Red));
                return;
            }
            println!("{}确认开始复习吗?(Y/N)", color::Fg(color::Green));
            loop {
                let mut confirmation = String::new();
                stdin().read_line(&mut confirmation).unwrap();
                match confirmation.trim() {
                    "Y" => {
                        println!(
                            "{}接下来将开始复习, 请按照控制台输出的单词输入其释义",
                            color::Fg(color::Green)
                        );
                        break;
                    }
                    "N" => return,
                    _ => println!(
                        "{}请输入'Y'来确认开始复习或者输入'N'退出复习",
                        color::Fg(color::Green)
                    ),
                }
            }
            loop {
                if let Some(current_word) = supervisor.get_current_word() {
                    match stdout().flush() {
                        Err(_e) => panic!("刷新输出失败"),
                        _ => {}
                    }
                    println!("{}> {}", color::Fg(color::White), current_word.name);
                    let mut input = String::new();
                    stdin().read_line(&mut input).unwrap();
                    let answer = input.trim();
                    match answer {
                        //todo
                        // _prompt 给出例句提示
                        // _skip 跳过当前复习的单词, 不给答案, 不算次数
                        // _cheat 直接给出答案, 不算次数
                        // _remove 将当前单词从表中直接移除单词
                        // _
                        "_exit" => break, //放弃复习
                        //输入单词
                        _ => {
                            if supervisor.exam(&answer) {
                                println!("{}回答正确", color::Fg(color::Green));
                                supervisor.set_word_reviewed_once();
                            } else {
                                // println!("{}回答内容是: {}", color::Fg(color::Red), answer);
                                println!(
                                    "{}回答错误，正确答案释义是: {}",
                                    color::Fg(color::Red),
                                    current_word.exp_cn
                                );
                            }
                            if supervisor.is_finished() {
                                //todo 复习完毕之后输出总结，包括成功次数，失败次数，共复习次数
                                println!("{}复习已完毕", color::Fg(color::Green));
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
        }else if  o.eq("remove"){
            if let Some(p) = matches.value_of("param"){
                let conn = establish_connection();
                let word_results: Vec<Word> = words
                    .filter(name.eq(p))
                    .limit(1)
                    .load::<Word>(&conn)
                    .expect("查询出错了");
                if word_results.len() > 0 {
                    let delete_result = diesel::delete(words.filter(name.eq(p))).execute(&conn);
                    match delete_result {
                        Ok(deleted_count)=>println!("成功删除{}条记录", deleted_count.to_string()),
                        _=>println!("删除失败")
                    }
                }else{
                    println!("单词不存在")
                }
            }
        }
        else {
            println!("{}无法识别的operation参数", color::Fg(color::Red));
        }
    }
}

pub fn create_word<'a>(conn: &SqliteConnection, name_field: &'a str, exp_cn_field: &'a str) {
    // use ragger_api::schema::words::dsl::*;
    let new_word = NewWord {
        name: name_field,
        exp_cn: exp_cn_field,
    };
    // println!("要插入的数据是 {:?}", &new_word);
    diesel::insert_into(words::table)
        .values(new_word)
        .execute(conn)
        .expect("插入数据失败");
    // let word_results = words.filter(status.eq(0)).limit(10).load::<Word>(conn).expect("查询出错了");
    // for word in word_results {
    //     println!("单词名称是:{}, 单词释义:{}", word.name, word.exp_cn);
    // }
    // println!("查询到了{}条数据", word_results.len());
}
