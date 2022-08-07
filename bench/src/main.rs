use std::time::Instant;

use rand::{seq::SliceRandom, Rng};
use rand_pcg::Mcg128Xsl64;
use redis::{Client, Commands};
use serde::{Deserialize, Serialize};
use ya_redis_derive::Redis;

const N_KEYS: usize = 100;
const N_READ_HEAVY: usize = 100;

static REDIS_ENDPOINT: &str = "redis://localhost:6379";

#[derive(Redis, Deserialize, Serialize)]
struct A {
    id: i64,
    name: String,
    score: u64,
    description: Option<String>,
}

impl A {
    fn gen<R: Rng>(rng: &mut R) -> A {
        static NAMES: &[&str] = &["abcdefg", "名無しの権兵衛", "最高にイケてる名前"];
        static DESC: &[Option<&str>] = &[
            None,
            Some("寿限無(じゅげむ)　寿限無(じゅげむ)　五劫(ごこう)のすりきれ　海砂利(かいじゃり)水魚(すいぎょ)の水行末(すいぎょうまつ)"),
            Some("Even though I walk through the valley of the shadow of death, I will fear no evil, for you are with me.")
        ];
        A {
            id: rng.gen(),
            name: NAMES.choose(rng).unwrap().to_string(),
            score: rng.gen(),
            description: DESC.choose(rng).unwrap().and_then(|s| Some(s.to_string())),
        }
    }
}

fn redis_ya_redis<R: Rng>(rng: &mut R) {
    let client = Client::open(REDIS_ENDPOINT).unwrap();
    let mut con = client.get_connection().unwrap();
    let mut keys = Vec::new();

    let start = Instant::now();
    for _ in 0..N_KEYS {
        let a = A::gen(rng);
        let key = format!("{}-{}", a.id, a.name);
        let _: bool = con.set(&key, &a).unwrap();
        let mut scores = 0;
        for _ in 0..N_READ_HEAVY {
            if let Some(a) = con.get::<_, Option<A>>(&key).unwrap() {
                scores += a.score as u128;
            }
        }
        assert_eq!(scores, a.score as u128 * N_READ_HEAVY as u128);
        keys.push(key);
    }
    let ms = start.elapsed().as_millis();
    println!(
        "redis ya_redis: set-get: total={}ms per_key={}ms",
        ms,
        ms / N_KEYS as u128
    );
    for key in keys {
        let _: bool = con.del(key).unwrap();
    }
}

fn redis_serde_json<R: Rng>(rng: &mut R) {
    let client = Client::open(REDIS_ENDPOINT).unwrap();
    let mut con = client.get_connection().unwrap();
    let mut keys = Vec::new();

    let start = Instant::now();
    for _ in 0..N_KEYS {
        let a = A::gen(rng);
        let key = format!("{}-{}", a.id, a.name);
        let s = serde_json::to_string(&a).unwrap();
        let _: bool = con.set(&key, s).unwrap();
        let mut scores = 0;
        for _ in 0..N_READ_HEAVY {
            if let Some(s) = con.get::<_, Option<String>>(&key).unwrap() {
                let a: A = serde_json::from_str(&s).unwrap();
                scores += a.score as u128;
            }
        }
        assert_eq!(scores, a.score as u128 * N_READ_HEAVY as u128);
        keys.push(key);
    }
    let ms = start.elapsed().as_millis();
    println!(
        "redis serde_json: set-get: total={}ms per_key={}ms",
        ms,
        ms / N_KEYS as u128
    );
    for key in keys {
        let _: bool = con.del(key).unwrap();
    }
}

fn main() {
    redis_serde_json(&mut Mcg128Xsl64::new(1));
    redis_ya_redis(&mut Mcg128Xsl64::new(1));
}