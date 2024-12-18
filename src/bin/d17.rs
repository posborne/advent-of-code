use std::{fmt::Display, ops::Deref, path::Path};

use aoc::input_lines;
use clap::Parser;

#[derive(Debug)]
#[repr(u8)]
pub enum Instruction {
    Adv = 0, // Division of A Register (numerator in A register) ...
    Bxl = 1, // Bitwise XOR of B register
    Bst = 2, // Combo Operand Module 8 -> B Register
    Jnz = 3, // Do nothing if A register is 0, If nonzero jump instruction pointer to value of its literal operand
    Bxc = 4, // Bitwise XOR of B and C (consume but ignore operand)
    Out = 5, // Output value of combo operand modulo 8
    Bdv = 6, // Division to B register (numerator stored in A register)
    Cdv = 7, // Division to C register
}

impl From<u8> for Instruction {
    fn from(value: u8) -> Self {
        if value <= Self::Cdv as u8 {
            unsafe { std::mem::transmute(value) }
        } else {
            panic!("{value} not a legal instruction");
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Opcode(u8);

impl Deref for Opcode {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct Machine {
    instruction_pointer: usize,
    output: Output,
    reg_a: isize,
    reg_b: isize,
    reg_c: isize,
}

#[derive(Debug, Clone)]
struct Output(Vec<u8>);

impl Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<String>>()
                .join(",")
        )
    }
}

impl Machine {
    fn div(&mut self, operand: u8) -> isize {
        let operand_combo_value = self.combo_value(operand);
        self.reg_a / (2_isize).pow(operand_combo_value as u32)
    }

    fn execute(&mut self, program: Vec<u8>) {
        while let (Some(&instruction_value), Some(&operand)) = (
            program.get(self.instruction_pointer),
            program.get(self.instruction_pointer + 1),
        ) {
            self.instruction_pointer += 2;
            let instruction = Instruction::from(instruction_value);
            match instruction {
                Instruction::Adv => self.reg_a = self.div(operand),
                Instruction::Bxl => self.reg_b ^= operand as isize,
                Instruction::Bst => self.reg_b = self.combo_value(operand) % 8,
                Instruction::Jnz => {
                    if self.reg_a != 0 {
                        self.instruction_pointer = operand as usize;
                    }
                }
                Instruction::Bxc => self.reg_b ^= self.reg_c,
                Instruction::Out => self.output.0.push((self.combo_value(operand) % 8) as u8),
                Instruction::Bdv => self.reg_b = self.div(operand),
                Instruction::Cdv => self.reg_c = self.div(operand),
            }
        }
    }

    fn combo_value(&self, operand: u8) -> isize {
        match operand {
            0 | 1 | 2 | 3 => operand as isize,
            4 => self.reg_a,
            5 => self.reg_b,
            6 => self.reg_c,
            7 => panic!("Illegal Combo operand value 7!"),
            _ => panic!("Operands can only be 3 bits in size!"),
        }
    }
}

fn parse_reg<I: Iterator<Item = String>>(lines: &mut I) -> anyhow::Result<isize> {
    let line = lines
        .next()
        .ok_or_else(|| anyhow::anyhow!("Not enough lines"))?;
    let value_str = line
        .split(":")
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("Failed to parse reg value"))?;
    Ok(value_str.trim().parse::<isize>()?)
}

fn parse_input<P: AsRef<Path>>(path: P) -> anyhow::Result<(Machine, Vec<u8>)> {
    let mut lines = input_lines(path)?;
    let reg_a = parse_reg(&mut lines)?;
    let reg_b = parse_reg(&mut lines)?;
    let reg_c = parse_reg(&mut lines)?;
    let _ = lines.next();
    let program = lines
        .next()
        .unwrap()
        .split(":")
        .nth(1)
        .unwrap()
        .trim()
        .split(",")
        .map(|opcode| opcode.parse::<u8>().unwrap())
        .collect();

    let machine = Machine {
        instruction_pointer: 0,
        output: Output(Vec::new()),
        reg_a,
        reg_b,
        reg_c,
    };
    Ok((machine, program))
}

#[derive(Parser)]
struct Cli {
    #[clap(short, long, default_value = "d17.txt")]
    input: String,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    println!("Input: {}", cli.input);
    let (mut machine, program) = parse_input(&cli.input)?;
    machine.execute(program);
    println!("Output: {}", machine.output);
    Ok(())
}
