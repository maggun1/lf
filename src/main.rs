use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::Arc;
use std::thread;
use std::time::Instant;
use clap::{Arg, Command};
use serde_json;

fn main() {
    let matches = Command::new("lf")
        .arg(Arg::new("filename")
            .num_args(1)
            .index(1)
            .required(true))
        .arg(Arg::new("threads")
            .short('t')
            .long("threads")
            .num_args(1)
            .default_value("1"))
        .get_matches();

    let start_time = Instant::now();
    let (tx, rv) = std::sync::mpsc::channel::<HashMap<char, usize>>();

    let filename = matches.get_one::<String>("filename").unwrap();
    let threads = matches.get_one::<String>("threads").and_then(|t| t.parse::<usize>().ok()).unwrap();

    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
    let lines = Arc::new(reader.lines().map(|l| l.unwrap()).collect::<Vec<String>>());

    let chunk_size = (lines.len() as f32 / threads as f32).ceil() as usize;

    let mut handles = vec![];
    for i in 0..threads {
        let start = i * chunk_size;
        let end = std::cmp::min(start + chunk_size, lines.len());

        let lines = lines.clone();
        let tx = tx.clone();

        let handle = thread::spawn(move || {
            let mut hashmap: HashMap<char, usize> = HashMap::new();
            for line in &lines[start..end] {
                for c in line.chars() {
                    if c.is_alphabetic() {
                        hashmap.entry(c).or_insert(0);
                        hashmap.entry(c).and_modify(|v| *v += 1);
                    }
                }
            }

            tx.send(hashmap).unwrap();
        });
        handles.push(handle);
    }
    drop(tx);

    for handle in handles {
        handle.join().unwrap();
    }

    let mut final_hashmap: HashMap<char, usize> = HashMap::new();
    for hashmap in rv.iter() {
        for (k, v) in hashmap {
            final_hashmap.entry(k)
                .and_modify(|val| *val += v)
                .or_insert(v);
        }
    }

    let end_time = Instant::now();
    let duration = end_time.duration_since(start_time);

    let result = serde_json::json!({
        "elapsed": format!("{:.3}s", duration.as_secs_f32()),
        "result": final_hashmap,
    });

    println!("{}", serde_json::to_string_pretty(&result).unwrap());
}