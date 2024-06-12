use clap::{value_parser, Arg, Command};
use reqwest;
use serde_json::Value;
use std::fs::File;
use std::io::Write;
use std::{collections::BTreeMap, fmt::Debug};
use url::Url;
const BASE_URL: &str = "https://yt.lemnoslife.com";

struct ParseError {
    message: String,
}

impl Debug for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

async fn send_request<T: AsRef<str>>(id: T) -> Result<Value, reqwest::Error> {
    let resp: Value = reqwest::get(format!(
        "{BASE_URL}/videos?part=chapters&id={}",
        id.as_ref(),
    ))
    .await?
    .json()
    .await?;
    if let Value::Object(err) = &resp["error"] {
        println!("{err:#?}");
    }
    return Ok(resp);
}

fn parse_response(resp: Value) -> Result<BTreeMap<u64, String>, ParseError> {
    if let Some(items) = resp["items"].as_array() {
        if let Some(chapters) = items[0]["chapters"]["chapters"].as_array() {
            return Ok(chapters
                .into_iter()
                .map(|val| {
                    (
                        val["time"].as_u64().unwrap(),
                        val["title"].as_str().unwrap().to_owned(),
                    )
                })
                .collect());
        }
    }
    return Err(ParseError {
        message: "Parsing error!".to_owned(),
    });
}
fn cli() -> Command {
    Command::new("chap")
        .about("your local library assistant")
        .version("0.0.1")
        .arg_required_else_help(true)
        .arg(
            Arg::new("url")
                .required(true)
                .index(1)
                .value_parser(value_parser!(String)),
        )
        .arg(
            Arg::new("dest")
                .required(true)
                .index(2)
                .value_parser(value_parser!(String)),
        )
}
#[tokio::main]
async fn main() {
    let matches = cli().get_matches();
    let url = Url::parse(matches.get_one::<String>("url").unwrap()).unwrap();
    let video_id: String;
    if let Some(_video_id) = url.query_pairs().find(|(key, _)| key == "v") {
        video_id = _video_id.1.into();
    } else {
        video_id = "".into();
    }
    let timestamps = parse_response(send_request(video_id).await.unwrap()).unwrap();
    let mut content = ";FFMETADATA1\n".to_string();
    for (time, title) in timestamps.into_iter() {
        match time {
            0 => content.push_str(&format!(
                "[CHAPTER]\nTIMEBASE=1/10000000\nSTART={time}\ntitle={title}\n"
            )),
            _ => {
                let x = time * 10000000;
                content.push_str(&format!(
                    "END={x}\n[CHAPTER]\nTIMEBASE=1/10000000\nSTART={x}\ntitle={title}\n"
                ))
            }
        }
    }
    let dest = matches.get_one::<String>("dest").unwrap();
    let mut file = File::create(dest).unwrap();
    file.write(content.as_bytes()).unwrap();
}
