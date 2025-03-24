// Copyright (c) 2024-2025 Antmicro <www.antmicro.com>
// SPDX-License-Identifier: Apache-2.0

use std::fs::File;
use std::io::{self, BufRead};

// Currently only care about time unit
#[derive(Debug)]
pub struct Spef {
    pub t_unit: f64,  // For time units (e.g., PS, NS)
}

impl Spef {
    fn new() -> Self {
        Spef {
            t_unit: 1.0
        }
    }
}

fn parse_time(input: &str) -> f64 {
    let parts: Vec<&str> = input.split_whitespace().collect();
    
    if parts.len() == 2 {
        let unit = match parts[1] {
            "NS" => 0.000000001,
            "PS" => 0.000000000001,
            _ => panic!("Unknown time unit")
        };

        if let Ok(scale) = parts[0].parse::<f64>() {
            return scale * unit;
        }
    }
    
    panic!("Unable to parse time unit")
}

pub fn parse_spef(file_path: &str) -> io::Result<Spef> {
    let file = File::open(file_path)?;
    let reader = io::BufReader::new(file);

    let mut header = Spef::new();

    for line in reader.lines().filter_map(Result::ok) {
        let line = line.trim();

        if line.is_empty() {
            continue;
        }
        
        if let Some(line) = line.split('*').last() {
            if line.starts_with("T_UNIT") {
                header.t_unit = parse_time(&line[6..].trim());

                // As we only care for T_UNIT for now, we can quit here
                break;
            }
        }
    }

    Ok(header)
}