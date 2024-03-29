#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;

#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate serde;
extern crate twitter_stream;

use regex::Regex;
use rocket::response::content;
use rocket::State;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use twitter_stream::rt::{self, Future, Stream};
use twitter_stream::{Token, TwitterStreamBuilder};

#[derive(Serialize, Deserialize, Debug)]
struct Tweet {
    text: String,
}

enum Sentinment {
    Positive,
    Neutral,
    Negative,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Mood {
    keyword: String,
    positive_count: u64,
    neutral_count: u64,
    negative_count: u64,
}

impl Mood {
    fn new(keyword: &str) -> Mood {
        Mood {
            keyword: keyword.to_string(),
            positive_count: 0,
            neutral_count: 0,
            negative_count: 0,
        }
    }

    fn update(&mut self, sentiment: Sentinment) {
        match sentiment {
            Sentinment::Positive => self.positive_count += 1,
            Sentinment::Neutral => self.neutral_count += 1,
            Sentinment::Negative => self.negative_count += 1,
        }
    }

    fn clear(&mut self) {
        self.positive_count = 0;
        self.neutral_count = 0;
        self.negative_count = 0;
    }
}

impl fmt::Display for Mood {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: +:{}, ~:{} -:{}",
            self.keyword, self.positive_count, self.neutral_count, self.negative_count
        )
    }
}

lazy_static! {
    static ref POSITIVE_REGEX : Regex = Regex::new(r"(happy|yay|joy|good)").unwrap();
    // static ref neutral_regex: Regex = Regex::new(r"(happy|yay|joy|good)").unwrap();
    static ref NEGATIVE_REGEX: Regex = Regex::new(r"(sad|woe|bad|oh noes|the worst)").unwrap();
    static ref KEYWORDS : Vec<&'static str> = vec!["twitter", "facebook", "google", "travel", "art", "music", "photography", "love", "fashion", "food"];
}

fn main() {
    // Attach to some stream and print the text of all the tweets
    let token = Token::new(
        fetch_env_var("API_KEY"),       //"consumer_key",
        fetch_env_var("API_SECRET"),    //"consumer_secret",
        fetch_env_var("ACCESS_TOKEN"),  // "access_key",
        fetch_env_var("ACCESS_SECRET"), //"access_secret",
    );

    let (tx, rx): (Sender<Tweet>, Receiver<Tweet>) = mpsc::channel();

    let future = TwitterStreamBuilder::filter(token)
        .track(Some(
            "twitter, facebook, google, travel, art, music, photography, love, fashion, food",
        ))
        .listen()
        .unwrap()
        .flatten_stream()
        .for_each(move |json| {
            let tweet: Result<Tweet, _> = serde_json::from_str(&json);
            if let Ok(ok_tweet) = tweet {
                let send_result = tx.send(ok_tweet);
                if let Err(error) = send_result {
                    panic!(error);
                }
            }
            Ok(())
        })
        .map_err(|e| eprintln!("error: {}", e));

    let mut moods: Vec<Mood> = Vec::with_capacity(KEYWORDS.len());
    KEYWORDS.iter().for_each(|word| moods.push(Mood::new(word)));
    thread::spawn(move || {
        println!("Listening for tweets...");

        let current_json: Arc<Mutex<String>> = Arc::new(Mutex::new("".to_string()));
        let json = current_json.clone();

        thread::spawn(move || {
            let mut every_nth = 0u64;
            for msg in rx {
                every_nth += 1;
                update_sentiments(&msg.text, &mut moods);
                if every_nth % 13 == 0 {
                    if let Ok(mut locked_json) = current_json.try_lock() {
                        *locked_json =
                            serde_json::to_string(&moods).expect("could not serialize moods");
                    }
                }

                if every_nth == 50_000 {
                    clear_sentiments(&mut moods);
                    every_nth = 0;
                }
            }
        });

        thread::spawn(move || {
            rocket::ignite()
                .mount("/", routes![index, sentiment_handler, get_wasm, get_js])
                .manage(json)
                .launch();
        });
    });
    rt::run(future);
}

fn fetch_env_var(name: &str) -> String {
    std::env::var(name).expect(&format!("Must configure {}", name))
}

fn update_sentiments(tweet_text: &str, current_sentiments: &mut Vec<Mood>) {
    for mood in current_sentiments.iter_mut() {
        if tweet_text.contains(&mood.keyword) {
            mood.update(classify(tweet_text));
        }
    }
}

fn clear_sentiments(sentiments: &mut Vec<Mood>) {
    for mood in sentiments.iter_mut() {
        mood.clear();
    }
}

fn classify(text: &str) -> Sentinment {
    if POSITIVE_REGEX.is_match(text) {
        return Sentinment::Positive;
    }
    if NEGATIVE_REGEX.is_match(text) {
        return Sentinment::Negative;
    }
    Sentinment::Neutral
}

#[get("/current")]
fn sentiment_handler(mood_state: State<Arc<Mutex<String>>>) -> content::Json<String> {
    let arc = mood_state.inner();
    if let Ok(locked_string) = arc.lock() {
        return content::Json(locked_string.to_string());
    }
    content::Json("[]".to_string())
}

#[get("/")]
fn index() -> content::Html<&'static str> {
    let result = include_str!("../web/stream-plotter/www/index.html");
    content::Html(result)
}

#[get("/plot.wasm")]
fn get_wasm() -> &'static [u8] {
    let result = include_bytes!("../web/stream-plotter/www/pkg/stream_plotter_bg.wasm");
    result
}

#[get("/stream_plotter.js")]
fn get_js() -> content::JavaScript<&'static str> {
    let result = include_str!("../web/stream-plotter/www/pkg/stream_plotter.js");
    content::JavaScript(result)
}
