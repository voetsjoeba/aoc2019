// vim: set ai et ts=4 sts=4 sw=4:

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
#[derive(PartialEq, Eq, Debug)]
pub enum ParamMode {
    Address,
    Immediate,
}
pub struct Instruction {
    value: i64,
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

pub struct CPU<IF,OF>
where
    IF: Fn() -> i64,
    OF: Fn(i64) -> (),
{
    pc: usize,
    mem: Vec<i64>,
    input_function: IF,
    output_function: OF,
}
impl<IF,OF> CPU<IF,OF>
where
    IF: Fn() -> i64,
    OF: Fn(i64) -> ()
{
    pub fn new(data: &Vec<i64>, input_fn: IF, output_fn: OF) -> Self {
        Self {
            pc: 0usize,
            mem: data.clone(),
            input_function: input_fn,
            output_function: output_fn,
        }
    }
    pub fn run(&mut self) {
        loop {
            let instr = Instruction::new(self.mem[self.pc]);
            if self.execute(&instr) == Op::Halt {
                break;
            }
        }
    }
    pub fn execute(&mut self, instr: &Instruction) -> Op {
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

            Op::Input => { let input = self.read_input();
                           self.write_param(0, instr, input);
                           self.pc += 2;
                         },

            Op::Output => { let value = self.read_param(0, instr);
                            self.write_output(value);
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

            Op::Halt => { },
        }
        return op;
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
    fn read_input(&self) -> i64 {
        (self.input_function)()
    }
    fn write_output(&self, value: i64) {
        (self.output_function)(value);
    }
}
