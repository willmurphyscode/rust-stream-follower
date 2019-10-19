#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate serde;
extern crate twitter_stream;

use std::collections::HashMap;

use regex::Regex;
use serde::{Deserialize, Serialize};

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

struct Mood {
    positive_count: u64,
    neutral_count: u64,
    negative_count: u64,
}

lazy_static! {
    static ref POSITIVE_REGEX : Regex = Regex::new(r"(happy|yay|joy|good)").unwrap();
    // static ref neutral_regex: Regex = Regex::new(r"(happy|yay|joy|good)").unwrap();
    static ref NEGATIVE_REGEX: Regex = Regex::new(r"(sad|woe|bad|oh noes|the worst)").unwrap();
    static ref KEYWORDS : Vec<&'static str> = vec!["twitter", "facebook", "google", "travel", "art", "music", "photography", "love", "fashion", "food"];
}

fn main() {
    // TEMP - make sure I can parse tweets
    let b = include_str!("./sample_tweet.json");
    let tweet: Tweet = serde_json::from_str(b).unwrap();
    println!("{:?}", tweet);

    // Attach to some stream and print the text of all the tweets
    let token = Token::new(
        fetch_env_var("API_KEY"),       //"consumer_key",
        fetch_env_var("API_SECRET"),    //"consumer_secret",
        fetch_env_var("ACCESS_TOKEN"),  // "access_key",
        fetch_env_var("ACCESS_SECRET"), //"access_secret",
    );

    let mut current_sentiments: HashMap<&str, Mood> = HashMap::new();

    let future = TwitterStreamBuilder::filter(token)
        .track(Some(
            "twitter, facebook, google, travel, art, music, photography, love, fashion, food",
        ))
        .listen()
        .unwrap()
        .flatten_stream()
        .for_each(|json| {
            let tweet: Tweet = serde_json::from_str(&json).unwrap();
            println!("Text of tweet: {}", tweet.text);
            Ok(())
        })
        .map_err(|e| println!("error: {}", e));

    rt::run(future);
}

fn fetch_env_var(name: &str) -> String {
    std::env::var(name).expect(&format!("Must configure {}", name))
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
