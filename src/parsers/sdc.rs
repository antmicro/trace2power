// Copyright (c) 2024-2025 Antmicro <www.antmicro.com>
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::io::{self};

// Currently only care about time unit
#[derive(Debug)]
pub struct Sdc {
    pub clock_period: f64,
}

impl Sdc {
    fn new() -> Self {
        Sdc {
            clock_period: 0.0
        }
    }
}

pub fn parse_sdc(file_path: &str) -> io::Result<Sdc> {
    let mut variables_map = HashMap::new();

    let mut sdc = Sdc::new();

    let input = std::fs::read_to_string(file_path).unwrap();
    let sdc_content = sdcx::Parser::parse(&input, &file_path).unwrap();
    for command in sdc_content.commands {
        match command {
            sdcx::sdc::Command::CreateClock(command) => {
                let period_str = command.period.as_str();
                sdc.clock_period = if period_str.starts_with('$') {
                    *variables_map.get(&period_str[1..]).unwrap()
                } else {
                    period_str.parse().unwrap()
                }
            },
            sdcx::sdc::Command::Set(command) => {
                variables_map.insert(command.variable_name.to_string(), command.value.as_str().parse().unwrap_or_else(
                    |_| {
                        println!("Unable to parse SDC variable value {:?} -> {:?}", command.variable_name.as_str(), command.value.as_str()); 0.0
                    }));
            },
            _ => {}
        }
    }

    Ok(sdc)
}