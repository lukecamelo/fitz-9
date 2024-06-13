struct CPU {
    registers: [u8; 0x10],
    program_counter: usize,
    stack: [u16; 16],
    stack_pointer: usize,
    memory: [u8; 0x1000], // 4KB of memory
}

#[allow(dead_code)]
impl CPU {
    fn run(&mut self) {
        loop {
            let op_byte1 = self.memory[self.program_counter] as u16;
            let op_byte2 = self.memory[self.program_counter + 1] as u16;
            let opcode: u16 = op_byte1 << 8 | op_byte2;

            println!("opcode: {:04x}", opcode);
            // each opcode is 2 bytes long
            self.program_counter += 2;

            // let c = ((opcode & 0xF000) >> 12) as u8;
            let x = ((opcode & 0x0F00) >> 8) as u8;
            let y = ((opcode & 0x00F0) >> 4) as u8;
            let d = ((opcode & 0x000F) >> 0) as u8;
            let nnn = opcode & 0x0FFF;
            let kk = (opcode & 0x00FF) as u8;

            match opcode {
                0x0000 => {
                    return;
                }
                0x00E0 => { /* CLEAR SCREEN ! */ },
                0x00EE => { self.ret(); },
                0x1000..=0x1FFF => { self.jmp(nnn); },
                0x2000..=0x2FFF => { self.call(nnn); },
                0x3000..=0x3FFF => { self.se(x, kk); },
                0x4000..=0x4FFF => { self.sne(x, kk); },
                0x5000..=0x5FFF => { self.se(x, y); },
                0x6000..=0x6FFF => { self.ld(x, kk); },
                0x7000..=0x7FFF => { self.add(x, kk); },
                0x8000..=0x8FFF => match d {
                    0x4 => {
                        self.add_xy(x, y);
                    }
                    0x5 => {
                        self.sub(x, y);
                    }
                    0x7 => {
                        self.sub(y, x);
                    }
                    0xE => {
                        self.shl(x);
                    }
                    _ => {
                        todo!("opcode {:04x}", opcode);
                    }
                },
                _ => todo!("opcode {:04x}", opcode),
            }
        }
    }

    fn ld(&mut self, x: u8, kk: u8) {
        self.registers[x as usize] = kk;
    }

    fn se(&mut self, vx: u8, kk: u8) {
        if vx == kk {
            self.program_counter += 2;
        }
    }

    fn sne(&mut self, vx: u8, kk: u8) {
        if vx != kk {
            self.program_counter += 2;
        }
    }

    fn add(&mut self, vx: u8, kk: u8) {
        self.registers[vx as usize] += kk;
    }

    fn sub(&mut self, vx: u8, vy: u8) {
        println!("subtracting!");
        let arg1 = self.registers[vx as usize];
        let arg2 = self.registers[vy as usize];

        let (val, overflow) = arg1.overflowing_sub(arg2);

        self.registers[vx as usize] = val;
        println!("{}", self.registers[vx as usize]);

        if overflow {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
    }

    fn shl(&mut self, vx: u8) {
        let msb = self.registers[vx as usize] & 0x80;

        if msb == 1 {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }    

        self.registers[vx as usize] = self.registers[vx as usize] * 2;
    }

    fn call(&mut self, addr: u16) {
        let sp = self.stack_pointer;
        let stack = &mut self.stack;

        if sp > stack.len() {
            panic!("Stack overflow!");
        }

        stack[sp] = self.program_counter as u16;
        self.stack_pointer += 1;
        self.program_counter = addr as usize;
    }

    fn jmp(&mut self, addr: u16) {
        self.program_counter = addr as usize;
    }

    fn ret(&mut self) {
        if self.stack_pointer == 0 {
            panic!("Stack underflow");
        }

        self.stack_pointer -= 1;

        let call_addr = self.stack[self.stack_pointer];

        self.program_counter = call_addr as usize;
    }

    fn add_xy(&mut self, x: u8, y: u8) {
        let arg1 = self.registers[x as usize];
        let arg2 = self.registers[y as usize];

        let (val, overflow) = arg1.overflowing_add(arg2);
        self.registers[x as usize] = val;
        println!("adding! {}", self.registers[x as usize]);

        if overflow {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
    }

    fn multiply_xy(&mut self, x: u8, y: u8) {
        let arg1 = self.registers[x as usize];
        let arg2 = self.registers[y as usize];

        let (val, overflow) = arg1.overflowing_mul(arg2);
        self.registers[x as usize] = val;

        if overflow {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
    }
}

fn main() {
    // initialize a CPU
    let mut cpu = CPU {
        registers: [0; 0x10],
        program_counter: 0,
        stack: [0; 16],
        stack_pointer: 0,
        memory: [0; 0x1000],
    };

    // load u8 values into registers
    cpu.registers[0] = 30;
    cpu.registers[1] = 5;

    let _add_twice: [u8; 6] = [0x81, 0x25, 0x81, 0x25, 0x00, 0xEE];
    let sub_twice: [u8; 6] = [0x80, 0x15, 0x80, 0x15, 0x00, 0xEE];

    let mem = &mut cpu.memory;

    mem[0x100..0x106].copy_from_slice(&sub_twice);

    // call function at memory address 0x100
    mem[0x000] = 0x21;
    mem[0x001] = 0x00;

    // call function at memory address 0x100
    mem[0x002] = 0x21;
    mem[0x003] = 0x00;

    // halt!
    // mem[0x004] = 0x00;
    // mem[0x005] = 0x00;

    // Add value in register 1 to register 0
    // mem[0x100] = 0x80;
    // mem[0x101] = 0x14;

    // Add value in register 1 to register 0
    mem[0x004] = 0x80;
    mem[0x005] = 0x14;

    // halt!
    mem[0x006] = 0x00;
    mem[0x007] = 0x00;

    // perform the operation
    cpu.run();

    assert_eq!(cpu.registers[0], 15);

    println!("30 - (5 * 2) - (5 * 2) + 5 = {}", cpu.registers[0]);
}
