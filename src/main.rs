mod lang;
mod mutator;
mod nodes;
mod sequence;
mod walker;

use clap::{Parser, Subcommand};
use lang::LangDescription;
use mutator::*;
use nodes::reset_name_counter;
use rand::seq::SliceRandom;
use rand::Rng;
use sequence::Seq;
use std::fs;
use std::path::Path;
use std::process::Command;
use walker::Walker;

#[derive(Parser)]
#[command(name = "seqfuzz2")]
#[command(about = "SeqFuzz2: IR-based fuzzer for PHP")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze a single PHP file and output the SeqIR as PHP
    Analyze {
        /// Path to the PHP file to analyze
        file: String,
    },
    /// Convert seed PHP files to serialized SeqIR graphs
    Preload {
        /// Chunk number (0-indexed) for parallel processing
        chunk: usize,
    },
    /// Run the main fuzzing loop
    Fuzz,
    /// Prepare the seed corpus from php-src
    Prepare,
}

/// List of banned PHP functions that could be dangerous
const BANNED: &[&str] = &[
    "posix_kill",
    "chmod",
    "posix_setrlimit",
    "exec",
    "passthru",
    "system",
    "shell_exec",
    "popen",
    "proc_open",
    "unlink",
    "rmdir",
    "rename",
    "chown",
    "chgrp",
    "symlink",
    "eval",
    "create_function",
    "assert",
    "dl",
    "pcntl_exec",
    "mail",
    "pcntl_fork",
    "pcntl_signal",
];

fn is_bad_seed(seed: &str) -> bool {
    if let Ok(content) = fs::read_to_string(seed) {
        for term in BANNED {
            if content.contains(term) {
                return true;
            }
        }
    }
    false
}

fn load_pickle(path: &str) -> Option<Seq> {
    let data = fs::read(path).ok()?;
    bincode::deserialize(&data).ok()
}

fn save_pickle(path: &str, seq: &Seq) -> Result<(), Box<dyn std::error::Error>> {
    let data = bincode::serialize(seq)?;
    fs::write(path, data)?;
    Ok(())
}

fn random_hex(len: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..len)
        .map(|_| format!("{:02x}", rng.gen::<u8>()))
        .collect()
}

fn parse_phpt(content: &str, section: &str) -> String {
    if !content.contains(section) {
        return String::new();
    }
    let start_idx = content.find(section).unwrap() + section.len();
    let remaining = &content[start_idx..];
    // Find next section marker like --EXPECT-- or --SKIPIF-- etc.
    let re_pattern = "--";
    let end_idx = remaining[1..] // skip first char to avoid matching the section itself
        .find(re_pattern)
        .map(|i| i + 1)
        .unwrap_or(remaining.len());
    remaining[..end_idx].trim_matches('\n').to_string()
}

fn cmd_analyze(file: &str) {
    reset_name_counter();
    let lang = match LangDescription::load() {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to load language description: {}", e);
            return;
        }
    };

    let mut w = Walker::new_with_lang(lang, true);
    let (_success, env) = w.analyze(file);

    let mut output = Vec::new();
    env.output_env(&mut output);
    println!("{}", output.join("\n"));
}

fn cmd_preload(chunk: usize) {
    let lang = match LangDescription::load() {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to load language description: {}", e);
            return;
        }
    };

    let target_directory = "php_seeds";
    let seeds: Vec<String> = match fs::read_dir(target_directory) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .map(|e| e.path().to_string_lossy().to_string())
            .collect(),
        Err(e) => {
            eprintln!("Failed to read seed directory: {}", e);
            return;
        }
    };

    let chunks = 16;
    let chunk_size = (seeds.len() + chunks - 1) / chunks;
    let start = chunk * chunk_size;
    let end = std::cmp::min(start + chunk_size, seeds.len());

    if start >= seeds.len() {
        eprintln!("Chunk {} is out of range", chunk);
        return;
    }

    // Ensure graphs directory exists
    let _ = fs::create_dir_all("graphs");

    for seed in &seeds[start..end] {
        reset_name_counter();
        let mut w = Walker::new_with_lang(lang.clone(), false);
        let (success, tree) = w.analyze(seed);

        let graph_name = if success && tree.all_nodes.len() > 1 {
            format!("graphs/{}.bin", random_hex(10))
        } else {
            format!("graphs/{}.error.bin", random_hex(10))
        };

        if let Err(e) = save_pickle(&graph_name, tree) {
            eprintln!("Failed to save graph for {}: {}", seed, e);
        }
    }
}

fn cmd_fuzz() {
    let lang = match LangDescription::load() {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to load language description: {}", e);
            return;
        }
    };

    let target_directory = "graphs";
    let output_directory = "scripts";

    // Ensure output directory exists
    let _ = fs::create_dir_all(output_directory);

    let seeds: Vec<String> = match fs::read_dir(target_directory) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .map(|e| e.path().to_string_lossy().to_string())
            .filter(|p| !p.ends_with(".error.bin"))
            .collect(),
        Err(e) => {
            eprintln!("Failed to read graphs directory: {}", e);
            return;
        }
    };

    if seeds.is_empty() {
        eprintln!("No seed graphs found in {}", target_directory);
        return;
    }

    let mut rng = rand::thread_rng();
    let is_error = |x: &str| Path::new(&format!("{}.er", x)).exists();
    let is_trash = |x: &str| Path::new(&format!("{}.tr", x)).exists();

    let mut count = 0;

    loop {
        count += 1;
        if count == 10 {
            count = 0;
            // Clean up
            let _ = Command::new("git")
                .args([
                    "clean", "-fd", "-e", "scripts", "-e", "php_seeds", "-e", "php",
                    "-e", "/ramdisk", "-e", "/graphs", "-e", "/graphs2",
                ])
                .output();
        }

        let php_file = format!("{}/{}.php", output_directory, random_hex(10));

        let result: Result<(), Box<dyn std::error::Error>> = (|| {
            reset_name_counter();
            let strategy = [0, 1, 3][rng.gen_range(0..3)];
            let mut target: Option<Seq> = None;

            match strategy {
                3 => {
                    let seed = seeds.choose(&mut rng).unwrap();
                    target = load_pickle(seed);
                    if let Some(ref mut t) = target {
                        let num_splices = rng.gen_range(1..=8);
                        for _ in 0..num_splices {
                            let splice_seed = seeds.choose(&mut rng).unwrap();
                            if let Some(splice) = load_pickle(splice_seed) {
                                let reps = rng.gen_range(3..=32);
                                stochastic_splice_dataflow(t, &splice, reps);
                            }
                        }
                    }
                }
                0 => {
                    let seed = seeds.choose(&mut rng).unwrap();
                    target = load_pickle(seed);
                    if let Some(ref mut t) = target {
                        let num_splices = rng.gen_range(1..=4);
                        for _ in 0..num_splices {
                            let splice_seed = seeds.choose(&mut rng).unwrap();
                            if let Some(splice) = load_pickle(splice_seed) {
                                if !stochastic_splice_controlflow(t, &splice) {
                                    stochastic_splice_dataflow(t, &splice, 3);
                                }
                            }
                        }
                    }
                }
                1 => {
                    let num_trees = rng.gen_range(2..=8);
                    let trees: Vec<Seq> = (0..num_trees)
                        .filter_map(|_| {
                            let seed = seeds.choose(&mut rng)?;
                            load_pickle(seed)
                        })
                        .collect();
                    if !trees.is_empty() {
                        target = Some(rando_max(&trees));
                    }
                }
                _ => {}
            }

            if let Some(ref mut t) = target {
                // Randomly modify operations
                if rng.gen_ratio(1, 3) {
                    modify_operation(t, &lang);
                }

                let mut tmp = Vec::new();
                t.output_env(&mut tmp);
                let mut output = "<?php\n".to_string();
                output.push_str(&tmp.join("\n"));

                fs::write(&php_file, &output)?;
            }

            Ok(())
        })();

        if result.is_err() {
            continue;
        }

        if !Path::new(&php_file).exists() {
            continue;
        }

        // Execute with sanitizer
        let cwd = std::env::current_dir().unwrap_or_default();
        let full_path = cwd.join(&php_file);

        for i in 0..2 {
            let child = Command::new("bash")
                .args([
                    "./sanitize.sh",
                    full_path.to_str().unwrap_or(""),
                    &i.to_string(),
                ])
                .output();

            match child {
                Ok(_) => {}
                Err(_) => break,
            }
        }

        // Clean up
        if is_trash(&php_file) {
            let _ = fs::remove_file(format!("{}.tr", php_file));
        } else if !is_error(&php_file) {
            let _ = fs::remove_file(&php_file);
        }
    }
}

fn cmd_prepare() {
    // Clone php-src
    let _ = Command::new("git")
        .args(["clone", "https://github.com/php/php-src.git"])
        .output();

    // Load corpus
    let _ = Command::new("bash")
        .args(["./corpus_loader.sh", "php-src"])
        .output();

    let files = match fs::read_to_string("tmp_files.txt") {
        Ok(content) => {
            let _ = fs::remove_file("tmp_files.txt");
            content
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect::<Vec<_>>()
        }
        Err(_) => {
            eprintln!("Failed to read tmp_files.txt");
            return;
        }
    };

    let _ = fs::create_dir_all("php_seeds");

    for file in &files {
        let full_path = format!("php-src/{}", file);
        if is_bad_seed(&full_path) {
            continue;
        }
        if let Ok(code) = fs::read_to_string(&full_path) {
            let parsed = parse_phpt(&code, "--FILE--");
            if !parsed.is_empty() {
                let output_path = format!("php_seeds/{}", random_hex(10));
                let _ = fs::write(output_path, parsed);
            }
        }
    }
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Analyze { file } => cmd_analyze(&file),
        Commands::Preload { chunk } => cmd_preload(chunk),
        Commands::Fuzz => cmd_fuzz(),
        Commands::Prepare => cmd_prepare(),
    }
}
