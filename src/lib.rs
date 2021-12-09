#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate crypto;
extern crate uuid;
extern crate chrono;
extern crate serde;
extern crate termion;


pub mod models;
pub mod schema;

use diesel::prelude::*;
use dotenv::dotenv;
use std::env;
use std::collections::HashMap;
use std::collections::hash_map::RandomState;
use crypto::sha2::Sha256;
use crypto::digest::Digest;
use uuid::Uuid;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use models::Word;
use schema::words::dsl::*;
use rand::thread_rng;
use rand::seq::SliceRandom;
use std::cmp::min;


pub trait Searchable {
    fn search(&self, word: String) -> Result<Vec<String>, ()>;
}

pub trait Supervisable {
    //初始化
    fn initialize(size_o: Option<usize>) -> Self;
    //检查输入的释义是否和焦点单词的释义相同
    fn exam(&self, explanation: &str) -> bool;
    //向前迭代一个单词
    fn forward(&mut self);
    //后退一个单词
    fn backward(&mut self);
    //获取当前单词的内容
    fn get_current_word(&self) -> Option<&Word>;
    //是否已经迭代完毕
    fn is_finished(&self) -> bool;
    //将当前单词设置为已复习状态
    fn set_word_reviewed_once(&self);
    //
    fn is_reviewable(&self)->bool;
}

pub struct Supervisor {
    tasks: Vec<Word>,
    cursor: usize,
    conn: SqliteConnection,
}


impl Supervisable for Supervisor {
    fn initialize(size_o :Option<usize>) -> Supervisor {
        let conn = establish_connection();
        let all_results:Vec<Word> = words.filter(status.gt(-1)).load::<Word>(&conn).expect("查询出错了");
        let half_size;
        if let Some(size) = size_o {
             half_size = min(all_results.len() / 2, size);
        }else{
             half_size = all_results.len() / 2;
        }

        let mut rng =  thread_rng();
        let items = all_results.choose_multiple(&mut rng, half_size);
            let mut word_results = Vec::new();
            for item in items {
                word_results.push(item.clone());
            }
            return Supervisor {
                tasks: word_results,
                cursor: 0,
                conn,
            };
    }

    fn exam(&self, explanation: &str) -> bool {
        if let Some(current_word) = self.tasks.get(self.cursor) {
            let explanations: Vec<String> = serde_json::from_str(&current_word.exp_cn).unwrap();
            return explanations.contains(&String::from(explanation));
        }
        return false;
    }

    fn forward(&mut self) {
        self.cursor += 1;
    }

    fn backward(&mut self) {
        self.cursor -= 1;
    }


    fn get_current_word(&self) -> Option<&Word> {
        return self.tasks.get(self.cursor);
    }

    fn is_finished(&self) -> bool {
        return self.cursor == self.tasks.len() - 1;
    }

    fn set_word_reviewed_once(&self) {
        let current_word = self.tasks.get(self.cursor).expect(format!("程序出错,无法找到单词,游标位置:{}", self.cursor).as_str());
        //如果status <=3 那么就status
        if current_word.status < 3 {
            diesel::update(words.find(current_word.id)).set(status.eq(status + 1)).execute(&self
                .conn).unwrap();
        } else {
            diesel::update(words.find(current_word.id)).set(status.eq(-1)).execute(&self.conn).unwrap();
        }
        //todo 这里直接用unwrap不大合适，需要进行错误处理
    }

    fn is_reviewable(&self) -> bool {
        return self.tasks.len() > 0;
    }
}

#[derive(Serialize, Deserialize)]
struct YoudaoResponse {
    #[serde(rename = "returnPhrase")]
    return_phrase: Vec<String>,
    query: String,
    #[serde(rename = "errorCode")]
    error_code: String,
    l: String,
    #[serde(rename = "tSpeakUrl")]
    t_speak_url: String,
    web: Vec<YoudaoValues>,
    #[serde(rename = "requestId")]
    request_id: String,
    translation: Vec<String>,
    dict: YoudaoDict,
    basic: YoudaoBasic,
    #[serde(rename = "isWord")]
    is_word: bool,
    #[serde(rename = "speakUrl")]
    speak_url: String,

}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
struct YoudaoValues {
    value: Vec<String>,
    key: String,
}

#[derive(Serialize, Deserialize)]
struct YoudaoDict {
    url: String,
}

#[derive(Serialize, Deserialize)]
struct YoudaoWebDict {
    url: String,
}

#[derive(Serialize, Deserialize)]
struct YoudaoBasic {
    explains: Vec<String>,
}


pub struct YoudaoSearcher {
    config: YoudaoConfig,
}

pub struct YoudaoConfig {
    pub appkey: String,
    pub appsecret: String,
}


impl Searchable for YoudaoSearcher {
    fn search(&self, word: String) -> Result<Vec<String>, ()> {
        let params: HashMap<&str, String> = self.build_params_by_word(word.clone());
        let response_text = self.exec_query(params);
        // println!("接口返回数据是:{}", &response_text);
        let wrapped_youdao_response: Result<YoudaoResponse, _> = serde_json::from_str(response_text.as_str());

        match wrapped_youdao_response {
            Ok(youdao_response)=>{
                let values_iter = youdao_response.web.iter();
                for val in values_iter {
                    if word.eq(&val.key.to_lowercase()) {
                        return Ok(val.value.clone());
                    }
                }
            }
            // Err(erro)=> return vec![],
            Err(_) =>return Err(())
        }

        return Err(());
    }
}

impl YoudaoSearcher {
    pub fn new(config: YoudaoConfig) -> YoudaoSearcher {
        YoudaoSearcher {
            config,
        }
    }


    fn build_sign(salt: &str, word: &str, app_key: &str, cur_time: &str, app_sec: &str)
                  -> String {
        let mut base_str: String = String::from(app_key);
        base_str.push_str(word);
        base_str.push_str(salt);
        base_str.push_str(cur_time);
        base_str.push_str(app_sec);
        let mut sha = Sha256::new();
        sha.input_str(base_str.as_str());
        return sha.result_str();
    }

    fn build_params_by_word(&self, word: String) -> HashMap<&'static str, String, RandomState> {
        let mut map = HashMap::new();
        let salt = Uuid::new_v4().to_string();
        let cur_time = Utc::now().timestamp().to_string();
        map.insert("sign", YoudaoSearcher::build_sign(
            salt.as_str(),
            word.as_str(),
            //"2f3a48a702316da0",
            self.config.appkey.as_str(),
            cur_time.as_str(),
            //"gTzzXsjEpFX9wUd8Rrio5SXfgzlcqq56"
            self.config.appsecret.as_str(),
        ),
        );
        map.insert("q", word);
        map.insert("from", String::from("zh-CHS"));
        map.insert("to", String::from("en"));
        map.insert("appKey", String::from("2f3a48a702316da0"));
        map.insert("salt", salt);
        map.insert("signType", String::from("v3"));
        map.insert("curtime", cur_time);
        return map;
    }

    fn exec_query(&self, params: HashMap<&str, String>) -> String {
        let client = reqwest::blocking::Client::new();
        // let params = [("foo", "bar"),("barz", "quux")]
        let res = client.post("https://openapi.youdao.com/api").form(&params).send()
            .unwrap().text().unwrap();
        return res;
    }
}

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

// type Result<T> = std::result<T, CommonError>;
//
// #[derive(Debug, Clone)]
// struct CommonError;
//
// impl fmt::Desplay for CommonError{
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
//         write!(f, "出错了")
//     }
// }



