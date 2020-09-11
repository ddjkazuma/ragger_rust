#[macro_use]
extern crate diesel;

extern crate dotenv;
extern crate crypto;
extern crate serde;

pub mod models;
pub mod schema;

use diesel::prelude::*;
use dotenv::dotenv;

use std::collections::HashMap;
use std::collections::hash_map::RandomState;
use crypto::sha2::Sha256;
use crypto::digest::Digest;
use uuid::Uuid;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde::export::fmt::Debug;
use std::env;


#[derive(Serialize, Deserialize)]
struct YoudaoResponse{
    #[serde(rename="returnPhrase")]
    return_phrase: Vec<String>,
    query: String,
    #[serde(rename="errorCode")]
    error_code: String,
    l: String,
    #[serde(rename="tSpeakUrl")]
    t_speak_url: String,
    web: Vec<YoudaoValues>,
    #[serde(rename="requestId")]
    request_id: String,
    translation: Vec<String>,
    dict: YoudaoDict,
    basic: YoudaoBasic,
    #[serde(rename="isWord")]
    is_word: bool,
    #[serde(rename="speakUrl")]
    speak_url: String,

}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
struct YoudaoValues{
    value :Vec<String>,
    key: String,
}

#[derive(Serialize, Deserialize)]
struct YoudaoDict{
    url: String,
}

#[derive(Serialize, Deserialize)]
struct YoudaoWebDict{
    url: String,
}

#[derive(Serialize, Deserialize)]
struct YoudaoBasic{
    explains: Vec<String>,
}



pub struct YoudaoSearcher{
    config: YoudaoConfig,
}

pub struct YoudaoConfig{
    pub appkey: String,
    pub appsecret: String,
}

pub trait Searchable{
    fn search(&self, word: String)->Vec<String>;
}

impl Searchable for YoudaoSearcher{
    fn search(&self, word: String) -> Vec<String> {
        let params: HashMap<&str, String> = self.build_params_by_word(word.clone());
        let response_text = self.exec_query(params);
        let youdao_response:YoudaoResponse = serde_json::from_str(response_text.as_str()).unwrap();
        let values_iter = youdao_response.web.iter();
        for val in values_iter {
            if word.eq(&val.key.to_lowercase()) {
                return  val.value.clone()
            }
        }

        return vec![];
    }
}

impl YoudaoSearcher{
    pub fn new(config :YoudaoConfig)->YoudaoSearcher{
        YoudaoSearcher{
            config,
        }
    }


    fn build_sign(salt : &str, word :&str, app_key :&str, cur_time :&str, app_sec :&str)
                ->String{
        let mut base_str:String  = String::from(app_key);
        base_str.push_str(word);
        base_str.push_str(salt);
        base_str.push_str(cur_time);
        base_str.push_str(app_sec);
        let mut sha =  Sha256::new();
        sha.input_str(base_str.as_str());
        return sha.result_str()
    }

    fn build_params_by_word(&self, word :String) -> HashMap<&'static str, String, RandomState> {
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
            self.config.appsecret.as_str()
            )
        );
        map.insert("q", String::from("hello"));
        map.insert("from", String::from("zh-CHS"));
        map.insert("to", String::from("en"));
        map.insert("appKey", String::from("2f3a48a702316da0"));
        map.insert("salt", salt);
        map.insert("signType", String::from("v3"));
        map.insert("curtime", cur_time);
        return map;

    }

    fn exec_query(&self, params: HashMap<&str, String>)->String{
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
