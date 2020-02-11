// vim: set ai et ts=4 sts=4 sw=4:
use std::ops::{Index, IndexMut};
use std::collections::{VecDeque, HashMap};
use std::convert::TryFrom;
use std::fmt;

#[derive(PartialEq, Eq, Clone, Copy, Hash,  Debug)]
pub enum Op {
    Add,
    Mul,
    Input,
    Output,
    JumpIfTrue,
    JumpIfFalse,
    LessThan,
    Equals,
    ShiftRelativeBase,
    Halt,
}
impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Op::Add               => "ADD",
            Op::Mul               => "MUL",
            Op::Input             => "IN",
            Op::Output            => "OUT",
            Op::JumpIfTrue        => "JT",
            Op::JumpIfFalse       => "JF",
            Op::LessThan          => "LT",
            Op::Equals            => "EQ",
            Op::Halt              => "HLT",
            Op::ShiftRelativeBase => "SRB",
        })
    }
}
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum ParamMode {
    Address,
    Immediate,
    RelativeAddress,
}
impl TryFrom<i64> for ParamMode {
    type Error = String;
    fn try_from(val: i64) -> Result<Self, Self::Error>{
        match val {
            0 => Ok(ParamMode::Address),
            1 => Ok(ParamMode::Immediate),
            2 => Ok(ParamMode::RelativeAddress),
            _ => Err(format!("invalid parameter mode: {}", val))
        }
    }
}

pub struct Instruction {
    opcode: Op,
    num_params: usize,
    param_modes: Vec<ParamMode>,
}
impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.opcode)
    }
}
impl TryFrom<i64> for Instruction {
    type Error = String;
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value % 100 {
            1  => Self::try_make(Op::Add, 3, value),
            2  => Self::try_make(Op::Mul, 3, value),
            3  => Self::try_make(Op::Input, 1, value),
            4  => Self::try_make(Op::Output, 1, value),
            5  => Self::try_make(Op::JumpIfTrue, 2, value),
            6  => Self::try_make(Op::JumpIfFalse, 2, value),
            7  => Self::try_make(Op::LessThan, 3, value),
            8  => Self::try_make(Op::Equals, 3, value),
            9  => Self::try_make(Op::ShiftRelativeBase, 1, value),
            99 => Self::try_make(Op::Halt, 0, value),
            _    => Err(format!("unrecognized op code: {}", value % 100))
        }
    }
}
#[allow(dead_code)]
impl Instruction {
    pub fn try_make(opcode: Op, num_params: usize, param_modes_value: i64)
        -> Result<Self, <Self as TryFrom<i64>>::Error>
    {
        let mut param_modes = Vec::<ParamMode>::new();
        for i in 0..num_params {
            let val = (param_modes_value / 10i64.pow(2+i as u32)) % 10;
            param_modes.push(ParamMode::try_from(val)?);
        }
        Ok(Self { opcode, num_params, param_modes })
    }
    pub fn param_mode(&self, num: usize) -> ParamMode {
        self.param_modes[num]
    }
    pub fn size(&self) -> usize { // size in "bytes"/"words"
        self.num_params + 1
    }
}

#[derive(PartialEq, Eq, Debug, Copy, Clone, Hash)]
pub enum CpuState {
    Running,
    Halted,
    WaitIO,
}
impl fmt::Display for CpuState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            CpuState::Running => "Running",
            CpuState::Halted  => "Halted",
            CpuState::WaitIO  => "WaitIO",
        })
    }
}

pub struct Memory {
    initial_data: Vec<i64>,
    extra: HashMap<usize, i64>,
}
impl Memory {
    pub fn new(initial_data: Vec<i64>) -> Self {
        Self {
            initial_data,
            extra: HashMap::new(),
        }
    }
}
impl Index<usize> for Memory {
    type Output = i64;
    fn index(&self, addr: usize) -> &Self::Output {
        if addr < self.initial_data.len() {
            return &self.initial_data[addr];
        }
        match self.extra.get(&addr) {
            Some(x) => x,
            None    => &0,
        }
    }
}
impl IndexMut<usize> for Memory {
    fn index_mut(&mut self, addr: usize) -> &mut Self::Output {
        if addr < self.initial_data.len() {
            return &mut self.initial_data[addr];
        }
        if !self.extra.contains_key(&addr) {
            self.extra.insert(addr, 0);
        }
        self.extra.get_mut(&addr).unwrap()
    }
}

pub struct CPU
{
    pc: usize,
    mem: Memory,
    input_queue: VecDeque<i64>,
    output_queue: VecDeque<i64>,
    state: CpuState,
    relative_base: i64,
}
#[allow(dead_code)]
impl CPU
{
    pub fn new(program: &Vec<i64>) -> Self {
        Self {
            pc: 0usize,
            mem: Memory::new(program.clone()),
            input_queue: VecDeque::new(),
            output_queue: VecDeque::new(),
            state: CpuState::Halted,
            relative_base: 0,
        }
    }
    pub fn reset(&mut self, program: &Vec<i64>) -> &mut Self {
        self.pc = 0usize;
        self.mem = Memory::new(program.clone());
        self.input_queue.clear();
        self.output_queue.clear();
        self.state = CpuState::Halted;
        self.relative_base = 0;
        self
    }
    pub fn run(&mut self) -> &mut Self {
        // starts (or restarts) the CPU and runs as far as possible until halting or waiting for IO.
        self.state = CpuState::Running;
        while self.state == CpuState::Running {
            self.step();
        }
        return self;
    }
    pub fn is_halted(&self) -> bool {
        self.state == CpuState::Halted
    }
    pub fn get_state(&self) -> CpuState {
        self.state
    }
    pub fn step(&mut self) -> &mut Self {
        let instr = Instruction::try_from(self.mem[self.pc]).unwrap();
        self.execute(&instr);
        return self;
    }
    pub fn execute(&mut self, instr: &Instruction) {
        // can't execute anything if we're halted
        if self.state == CpuState::Halted {
            panic!("cannot execute instruction; CPU has halted");
        }
        match instr.opcode {
            Op::Add => { let arg1 = self.read_param(0, instr);
                         let arg2 = self.read_param(1, instr);
                         self.write_param(2, instr, arg1+arg2);
                         self.pc += 4;
                       },

            Op::Mul => { let arg1 = self.read_param(0, instr);
                         let arg2 = self.read_param(1, instr);
                         self.write_param(2, instr, arg1*arg2);
                         self.pc += 4;
                       },

            Op::Input => { if let Some(input) = self.input_queue.pop_front() {
                               self.write_param(0, instr, input);
                               self.pc += 2;
                               // if we were previously waiting for input, we should now switch back to Running
                               // (we may have been resumed after having been given new input to process)
                               self.state = CpuState::Running;
                           } else {
                               // no input yet; stay on the same instruction and move to the wait state to be resumed later
                               self.state = CpuState::WaitIO;
                           }
                         },

            Op::Output => { let value = self.read_param(0, instr);
                            self.output_queue.push_back(value);
                            self.pc += 2;
                          },

            Op::JumpIfTrue => { let value   = self.read_param(0, instr);
                                let jump_pc = self.read_param(1, instr);
                                self.pc = match value {
                                    0 => self.pc + 3,
                                    _ => jump_pc as usize,
                                }},

            Op::JumpIfFalse => { let value   = self.read_param(0, instr);
                                 let jump_pc = self.read_param(1, instr);
                                 self.pc = match value {
                                    0 => jump_pc as usize,
                                    _ => self.pc + 3,
                                 }},

            Op::LessThan => { let arg1 = self.read_param(0, instr);
                              let arg2 = self.read_param(1, instr);
                              self.write_param(2, instr, if arg1 < arg2 { 1 } else { 0 });
                              self.pc += 4;
                            },

            Op::Equals => { let arg1 = self.read_param(0, instr);
                            let arg2 = self.read_param(1, instr);
                            self.write_param(2, instr, if arg1 == arg2 { 1 } else { 0 });
                            self.pc += 4;
                          },

            Op::ShiftRelativeBase => { let arg1 = self.read_param(0, instr);
                                       self.relative_base += arg1;
                                       self.pc += 2;
                                     },

            Op::Halt => { self.state = CpuState::Halted; },
        }
    }
    fn read_param(&self, num: usize, instr: &Instruction) -> i64 {
        let param_value = self.mem[self.pc + 1 + num];
        let param_mode = instr.param_mode(num);
        match param_mode {
            ParamMode::Immediate       => param_value,
            ParamMode::Address         => self.mem[param_value as usize],
            ParamMode::RelativeAddress => self.mem[(self.relative_base + param_value) as usize]
        }
    }
    fn write_param(&mut self, num: usize, instr: &Instruction, value: i64) {
        let param_value = self.mem[self.pc + 1 + num];
        let param_mode = instr.param_mode(num);
        match param_mode {
            ParamMode::Immediate       => { panic!("invalid parameter mode for output value"); }
            ParamMode::Address         => { self.mem[param_value as usize] = value; },
            ParamMode::RelativeAddress => { self.mem[(self.relative_base + param_value) as usize] = value; },
        }
    }
    pub fn write_mem(&mut self, addr: i64, value: i64) -> &mut Self {
        // for external access to writing memory
        self.mem[addr as usize] = value;
        self
    }
    pub fn read_mem(&mut self, addr: i64) -> i64 {
        self.mem[addr as usize]
    }
    pub fn send_input(&mut self, input: i64) -> &mut Self{
        self.input_queue.push_back(input);
        return self;
    }
    pub fn send_input_iter(&mut self, iter: impl Iterator<Item=i64>) {
        self.input_queue.extend(iter);
    }
    pub fn send_input_string(&mut self, s: &str) {
        self.input_queue.extend(s.chars().map(|c| c as i64));
    }
    pub fn peek_input_first(&self) -> Option<i64> {
        self.input_queue.front().cloned()
    }
    pub fn peek_output_last(&self) -> Option<i64> {
        // returns the last value from the output queue (if any) without removing it
        self.output_queue.back().cloned()
    }
    pub fn consume_output_n(&mut self, n: usize) -> Option<Vec<i64>> {
        // remove and return the first N output values from the queue, if there at least that many.
        // otherwise, returns None.
        if self.output_queue.len() >= n {
            return Some(self.output_queue.drain(..n).collect());
        }
        return None;
    }
    pub fn consume_output(&mut self) -> Option<i64> {
        // pops a single value from the output queue, if any
        self.output_queue.pop_front()
    }
    pub fn consume_output_last(&mut self) -> Option<i64> {
        self.consume_output_all().into_iter().last()
    }
    pub fn consume_output_all(&mut self) -> Vec<i64> {
        let mut result = Vec::new();
        while let Some(x) = self.output_queue.pop_front() {
            result.push(x);
        }
        result
    }
}

pub struct Disas {
}
#[allow(dead_code)]
impl Disas {
    pub fn disassemble(program: &Vec<i64>) -> String {
        let mut result = String::new();

        let mut pc: usize = 0;
        while pc < program.len() {
            result += &format!("{:06X}  ", pc);
            if let Ok(instr) = Instruction::try_from(program[pc]) {
                result += &Self::disassemble_instr(program, pc, &instr);
                result += "\n";
                pc += instr.size();
            } else {
                // not a valid instruction, treat it as data
                result += &format!("{:-6} {:02X}\n", "", program[pc]);
                pc += 1;
            }
        }

        return result;
    }
    pub fn disassemble_instr(program: &Vec<i64>, pc: usize, instr: &Instruction) -> String {
        let mut result = format!("{:-6}", instr.to_string());
        if instr.num_params > 0 {
            result += " ";
            for n in 0..instr.num_params {
                let param_value = program[pc + 1 + n];
                result.push_str(&match instr.param_mode(n) {
                    ParamMode::Immediate       => Self::format_immediate(param_value),
                    ParamMode::Address         => format!("[{:02X}]", param_value),
                    ParamMode::RelativeAddress => format!("[base + {:02X}]", param_value),
                });
                if n < instr.num_params - 1 {
                    result += ", ";
                }
            }
        }
        return result;
    }
    fn format_immediate(val: i64) -> String {
        if val < 0 {
            format!("$-{:02X}", -val)
        } else {
            format!("${:02X}", val)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn io_states() {
        let mut cpu = CPU::new(&vec![3,0,4,0,99]); // reads an input and outputs it again
        cpu.run();

        // CPU should be paused waiting for input, staying on the same INPUT instruction
        assert_eq!(cpu.get_state(), CpuState::WaitIO);
        assert_eq!(cpu.consume_output_last(), None);

        // telling it to continue shouldn't help, it still needs some input to read
        cpu.run();
        assert_eq!(cpu.get_state(), CpuState::WaitIO);

        // now put some input on its queue, but don't tell it to continue doing anything yet;
        // should still be in the waiting state until we tell it to resume execution
        cpu.send_input(17);
        assert_eq!(cpu.get_state(), CpuState::WaitIO);
        assert_eq!(cpu.consume_output_last(), None);

        // now make the CPU retry the instruction where it left off (i.e. the input instr)
        cpu.step();
        assert_eq!(cpu.get_state(), CpuState::Running);

        // and let it run to completion, and check that it produced the same input value as output
        cpu.run();
        assert_eq!(cpu.get_state(), CpuState::Halted);
        assert_eq!(cpu.consume_output_all(), vec![17]);
    }
}
