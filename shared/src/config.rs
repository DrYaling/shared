use once_cell::sync::Lazy;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::prelude::*;
use std::sync::Arc;

static PARAMETERS: Lazy<Arc<BTreeMap<String, String>>> = Lazy::new(|| {
    let mut file = File::open("config.json").expect("config.json 文件不存在");
    let mut config_str = String::new();
    file.read_to_string(&mut config_str).expect("读取文件失败");
    Arc::new(serde_json::from_str::<BTreeMap<String, String>>(config_str.as_str()).unwrap())
});

pub fn get(key: &str)-> Option<String> {
    PARAMETERS.get(key).map(|v| v.clone() )
}

pub fn get_redis()-> redis::Connection {
    let mut r = redis::Client::open(PARAMETERS.get("redis_url").unwrap().as_ref()).and_then(|c| c.get_connection() ).unwrap();
    PARAMETERS.get("redis_password").map(|p| redis::cmd("AUTH").arg(p).execute(&mut r));
    r
}
