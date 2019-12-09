// vim: set ai et ts=4 sts=4 sw=4:
use std::collections::VecDeque;
use std::fmt;

#[derive(PartialEq, Eq, Debug)]
pub enum Op {
    Add,
    Mul,
    Input,
    Output,
    JumpIfTrue,
    JumpIfFalse,
    LessThan,
    Equals,
    Halt,
}
impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Op::Add => "Add",
            Op::Mul => "Mul",
            Op::Input => "Input",
            Op::Output => "Output",
            Op::JumpIfTrue => "JumpIfTrue",
            Op::JumpIfFalse => "JumpIfFalse",
            Op::LessThan => "LessThan",
            Op::Equals => "Equals",
            Op::Halt => "Halt",
        })
    }
}
#[derive(PartialEq, Eq, Debug)]
pub enum ParamMode {
    Address,
    Immediate,
}
pub struct Instruction {
    value: i64,
}
impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.opcode())
    }
}
impl Instruction {
    pub fn new(value: i64) -> Self {
        Self { value }
    }
    pub fn opcode(&self) -> Op {
        match self.value % 100 {
            1  => Op::Add,
            2  => Op::Mul,
            3  => Op::Input,
            4  => Op::Output,
            5  => Op::JumpIfTrue,
            6  => Op::JumpIfFalse,
            7  => Op::LessThan,
            8  => Op::Equals,
            99 => Op::Halt,
            _  => panic!("invalid opcode: {}", self.value % 100),
        }
    }
    pub fn param_mode(&self, num: u32) -> ParamMode {
        let val = (self.value / 10i64.pow(2+num)) % 10;
        match val {
            0 => ParamMode::Address,
            1 => ParamMode::Immediate,
            _ => panic!("unrecognized parameter mode: {}", val),
        }
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
            CpuState::Halted => "Halted",
            CpuState::WaitIO => "WaitIO",
        })
    }
}
pub struct CPU
{
    pc: usize,
    mem: Vec<i64>,
    input_queue: VecDeque<i64>,
    output_queue: VecDeque<i64>,
    state: CpuState,
}
impl CPU
{
    pub fn new(program: &Vec<i64>) -> Self {
        Self {
            pc: 0usize,
            mem: program.clone(),
            input_queue: VecDeque::new(),
            output_queue: VecDeque::new(),
            state: CpuState::Halted,
        }
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
    #[allow(dead_code)]
    pub fn get_state(&self) -> CpuState {
        self.state
    }
    pub fn step(&mut self) -> &mut Self {
        let instr = Instruction::new(self.mem[self.pc]);
        self.execute(&instr);
        return self;
    }
    pub fn execute(&mut self, instr: &Instruction) {
        // can't execute anything if we're halted
        if self.state == CpuState::Halted {
            panic!("cannot execute instruction; CPU has halted");
        }
        let op = instr.opcode();
        match op {
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

            Op::Halt => { self.state = CpuState::Halted; },
        }
    }
    fn read_param(&self, num: usize, instr: &Instruction) -> i64 {
        let param_value = self.mem[self.pc + 1 + num];
        let param_mode = instr.param_mode(num as u32);
        match param_mode {
            ParamMode::Address   => self.mem[param_value as usize],
            ParamMode::Immediate => param_value,
        }
    }
    fn write_param(&mut self, num: usize, instr: &Instruction, value: i64) {
        let param_value = self.mem[self.pc + 1 + num];
        let param_mode = instr.param_mode(num as u32);
        match param_mode {
            ParamMode::Address   => { self.mem[param_value as usize] = value; },
            ParamMode::Immediate => { panic!("invalid parameter mode for output value"); }
        }
    }
    pub fn send_input(&mut self, input: i64) -> &mut Self{
        self.input_queue.push_back(input);
        return self;
    }
    pub fn consume_output(&mut self) -> Option<i64> {
        // pops a single value from the output queue, if any
        self.output_queue.pop_front()
    }
    pub fn consume_output_last(&mut self) -> Option<i64> {
        // consumes the output and returns the last value
        let mut result: Option<i64> = None;
        while let Some(x) = self.output_queue.pop_front() {
            result = Some(x);
        }
        return result;
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
        assert_eq!(cpu.consume_output_last(), Some(17));
    }
}
