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

    for line in reader.lines() {
        let mut line = line?;

        if line.trim().is_empty() {
            continue;
        }
        
        line = line.split("*").last().unwrap().to_string();
        line = line.trim().to_string();

        if line.starts_with("T_UNIT") {
            let time_unit = line[6..].trim().to_string();
            header.t_unit = parse_time(&time_unit);

            // As we are only interested in the time unit, we can stop parsing after
            // extracting the needed information.
            break;
        }
    }

    Ok(header)
}