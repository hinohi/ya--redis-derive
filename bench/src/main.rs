use std::time::Instant;

use rand::{seq::SliceRandom, Rng};
use rand_pcg::Mcg128Xsl64;
use redis::{Client, Commands, Connection};
use serde::{Deserialize, Serialize};
use ya_redis_derive::Redis;

const N_KEYS: usize = 100;
const N_READ_HEAVY: usize = 10;

static REDIS_ENDPOINT: &str = "redis://localhost:6379";
static DRAGONFLY_ENDPOINT: &str = "redis://localhost:16379";

#[derive(Redis, Deserialize, Serialize)]
struct A {
    id: i64,
    name: String,
    score: u64,
    description: Option<String>,
}

#[derive(Redis)]
struct V(Vec<A>);

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

fn via_ya_redis<R: Rng>(server: &str, rng: &mut R, con: &mut Connection, n: usize) -> u128 {
    let mut keys = Vec::new();
    let start = Instant::now();
    for i in 0..N_KEYS {
        let a = (0..n).map(|_| A::gen(rng)).collect::<Vec<_>>();
        let key = format!("{}", i);
        let _: bool = con.set(&key, &V(a)).unwrap();
        let mut scores = 0;
        for _ in 0..N_READ_HEAVY {
            if let Some(v) = con.get::<_, Option<V>>(&key).unwrap() {
                for a in v.0 {
                    scores += a.score as u128;
                }
            }
        }
        assert!(scores > n as u128 * N_READ_HEAVY as u128 * 1000);
        keys.push(key);
    }
    let ms = start.elapsed().as_millis();
    println!(
        "{} ya_redis: n={} total={}ms per_key={}ms",
        server,
        n,
        ms,
        ms / N_KEYS as u128
    );
    for key in keys {
        let _: bool = con.del(key).unwrap();
    }
    ms
}

fn via_serde_json<R: Rng>(
    server: &str,
    rng: &mut R,
    con: &mut Connection,
    n: usize,
) -> (u128, usize) {
    let mut keys = Vec::new();
    let start = Instant::now();
    let mut bytes = 0;
    for i in 0..N_KEYS {
        let a = (0..n).map(|_| A::gen(rng)).collect::<Vec<_>>();
        let key = format!("{}", i);
        let s = serde_json::to_string(&a).unwrap();
        bytes += s.as_bytes().len();
        let _: bool = con.set(&key, s).unwrap();
        let mut scores = 0;
        for _ in 0..N_READ_HEAVY {
            if let Some(s) = con.get::<_, Option<String>>(&key).unwrap() {
                let v: Vec<A> = serde_json::from_str(&s).unwrap();
                for a in v {
                    scores += a.score as u128;
                }
            }
        }
        assert!(scores > n as u128 * N_READ_HEAVY as u128 * 1000);
        keys.push(key);
    }
    let ms = start.elapsed().as_millis();
    println!(
        "{} serde_json: n={} total={}ms per_key={}ms",
        server,
        n,
        ms,
        ms / N_KEYS as u128
    );
    for key in keys {
        let _: bool = con.del(key).unwrap();
    }
    (ms, bytes / N_KEYS)
}

#[derive(Default)]
struct StaticRecord {
    redis_ya_redis_ms: u128,
    redis_serde_json_ms: u128,
    dragonfly_ya_redis_ms: u128,
    dragonfly_serde_json_ms: u128,
    json_bytes: usize,
}

impl StaticRecord {
    fn dump(&self) {
        println!(
            "{},{},{},{},{}",
            self.json_bytes,
            self.redis_ya_redis_ms,
            self.redis_serde_json_ms,
            self.dragonfly_ya_redis_ms,
            self.dragonfly_serde_json_ms
        );
    }
}

fn main() {
    let mut records = Vec::new();
    for n in [1, 4, 16, 64, 256, 1024, 4096] {
        let mut r = StaticRecord::default();
        let client = Client::open(REDIS_ENDPOINT).unwrap();
        let mut con = client.get_connection().unwrap();
        r.redis_ya_redis_ms = via_ya_redis("redis", &mut Mcg128Xsl64::new(1), &mut con, n);
        let (ms, b) = via_serde_json("redis", &mut Mcg128Xsl64::new(1), &mut con, n);
        r.redis_serde_json_ms = ms;
        r.json_bytes = b;

        let client = Client::open(DRAGONFLY_ENDPOINT).unwrap();
        let mut con = client.get_connection().unwrap();
        r.dragonfly_ya_redis_ms = via_ya_redis("dragonfly", &mut Mcg128Xsl64::new(1), &mut con, n);
        r.dragonfly_serde_json_ms =
            via_serde_json("dragonfly", &mut Mcg128Xsl64::new(1), &mut con, n).0;
        records.push(r);
    }
    for r in records {
        r.dump();
    }
}
