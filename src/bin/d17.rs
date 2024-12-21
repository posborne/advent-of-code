use std::{ops::Deref, path::Path};

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
    reg_a: isize,
    reg_b: isize,
    reg_c: isize,
}

fn print_output(out: &[u8]) {
    println!("{}", out.iter().map(|i| i.to_string()).collect::<Vec<String>>().join(","));
}

impl Machine {
    fn div(&mut self, operand: u8) -> isize {
        let operand_combo_value = self.combo_value(operand);
        self.reg_a / (1 << operand_combo_value)
    }

    fn execute(&mut self, program: &[u8]) -> Vec<u8> {
        let mut output: Vec<u8> = Vec::new();
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
                Instruction::Out => output.push((self.combo_value(operand) % 8) as u8),
                Instruction::Bdv => self.reg_b = self.div(operand),
                Instruction::Cdv => self.reg_c = self.div(operand),
            }
        }
        output
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

fn part1() -> anyhow::Result<()> {
    let cli = Cli::parse();
    println!("== Part 1 ==");
    println!("Input: {}", cli.input);
    let (mut machine, program) = parse_input(&cli.input)?;
    let out = machine.execute(&program);
    print_output(&out);
    println!("");
    Ok(())
}

// So, in part 2 we need to tweak register A so that when the program is
// run it outputs the program itself again (programception).
//
// Let's look at our input program to see what we can do as far as working
// backwards...
//
// > Program: 2,4,1,2,7,5,4,1,1,3,5,5,0,3,3,0
//
// 2,4 => bst 4: b = a % 8 (...)
// 1,2 => bxl 2: b = b ^ 2
// 7,5 => cdv 5: c = b / 5
// 4,1 => bxc 1: b = b ^ c
// 1,3 => bxl 3: b = b ^ 3
// 5,5 => out 5: out (b % 8) -> 1: b % 8 must equal 2
// 0,3 => adv 3: a = a / 3
// 3.0 => jnz 0: loop if a != 0
//
// Based on these inputs, we see a few things:
// 1. The only looping is done at the end of the program until a is zero
// 2. The only assigned to a is a = a / (1 << 3) so we are going to be
//    essentially right shifting off bytes over time.
// 3. Given this, for each output value we just need to solve for
//    the lowest bits of a in chunks and then reconstitute the final value.

fn part2() -> anyhow::Result<()> {
    let cli = Cli::parse();
    println!("== Part 2 ==");
    println!("Input: {}", cli.input);
    let (original_machine, program) = parse_input(&cli.input)?;
    let mut saved: Vec<isize> = Vec::new();

    let run_with_a = |a: isize| {
        let mut machine = original_machine.clone();
        machine.reg_a = a;
        machine.execute(&program)
    };

    // brute force for an initial value for the lowest bits of a
    for a in 0..1024 {
        let out = run_with_a(a);
        if out[0] == program[0] && out[1] == program[1] {
            saved.push(a);
        }
    }

    // NOTE: I got part of this solution watching a youtube video when
    // I was a bit stuck and frustrated with this one... still fully
    // working on grokking the logic with the exact shifts here...
    //
    // See  https://www.youtube.com/watch?v=OjFGKL54yJQ
    for pos in 1..program.len() {
        let mut next = Vec::new();
        for candidate in saved {
            for bit in 0..8 {
                // our next candidate for a is one of the previous values of a that
                // worked for the lower bits OR'd with each combination of 8 bits
                // left shifted into by the required position for this pass
                let a_candidate = (bit << (7 + 3 * pos)) | candidate;
                let out = run_with_a(a_candidate);
                if out.len() > pos && out[pos] == program[pos] {
                    next.push(a_candidate);
                }
            }
        }
        saved = next;
    }

    let min = *saved.iter().min().unwrap();
    println!("Program: {program:?}");
    println!("Output:  {:?}", run_with_a(min));
    println!("Min: {min}");

    Ok(())
}

fn main() -> anyhow::Result<()> {
    part1()?;
    part2()?;
    Ok(())
}
