use rand::random;

const FONTSET_SIZE: usize = 80;

const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const NUM_REG: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;

pub struct Emu {
    pc: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_reg: [u8; NUM_REG],
    i_reg: u16,
    sp: u16,
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_KEYS],
    dt: u8,
    st: u8,
}

// The first 512 bytes were allocated to the original interpreter. Most programs as a result
// started at 0x200.
const START_ADDR: u16 = 0x200;

impl Emu {
    pub fn new() -> Self {
        let mut new_emu = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REG],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
        };

        // Before we return our new instance, load the fontset into RAM
        new_emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        new_emu
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_REG];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    // Runs the whole damn thing
    pub fn tick(&mut self) {
        // Fetch instruction to run
        let op = self.fetch();

        // Decode & Execute decoded instruction
        self.execute(op);
    }

    fn fetch(&mut self) -> u16 {
        // TODO: Fetch the next instruction to perform
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;

        // Convert our two byte instructions into a single 16-bit variable to be parsed
        let op = (higher_byte << 8) | lower_byte;

        // Increment our counter for the next tick
        self.pc += 2;

        op
    }

    fn execute(&mut self, op: u16) {
        // Get each digit for easier matching
        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 4;
        let digit4 = op & 0x000F;

        // Get "nibbles" for repeated use with instructions
        let x = ((op & 0x0F00) >> 8) as usize; // Second nibble (register X)
        let y = ((op & 0x00F0) >> 4) as usize; // Third nibble (register Y)

        let n = (op & 0x000F) as u8; // Fourth nibble
        let nn = (op & 0x00FF) as u8; // Lower byte
        let nnn = op & 0x0FFF; // Lower 12 bits (address)

        match (digit1, digit2, digit3, digit4) {
            // NOP
            (0, 0, 0, 0) => (),

            // CLS
            (0, 0, 0xE, 0) => self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT],

            // RET
            (0, 0, 0xE, 0xE) => self.pc = self.pop(),

            // JP addr
            (1, _, _, _) => self.pc = nnn,

            // CALL addr
            (2, _, _, _) => {
                // self.sp += 1;

                self.push(self.pc);

                self.pc = nnn;
            }

            // SE Vx, byte
            (3, _, _, _) => {
                if self.v_reg[x] == nn {
                    self.pc += 2;
                }
            }

            // SNE Vx, byte
            (4, _, _, _) => {
                if self.v_reg[x] != nn {
                    self.pc += 2;
                }
            }

            // SE Vx, Vy
            (5, _, _, 0) => {
                if self.v_reg[x] == self.v_reg[y] {
                    self.pc += 2;
                }
            }

            // LD Vx, byte
            (6, _, _, _) => {
                self.v_reg[x] = nn;
            }

            // ADD Vx, byte
            (7, _, _, _) => {
                // We do wrapping add here specifically to mimic the CPU (Rust will panic by
                // default)
                self.v_reg[x] = self.v_reg[x].wrapping_add(nn);
            }

            // LD Vx, Vy
            (8, _, _, 0) => {
                self.v_reg[x] = self.v_reg[y];
            }

            // OR Vx, Vy
            (8, _, _, 1) => {
                // The same as self.v_reg[x] = self.v_reg[x] | self.v_reg[y];
                self.v_reg[x] |= self.v_reg[y];
            }

            // AND Vx, Vy
            (8, _, _, 2) => {
                self.v_reg[x] &= self.v_reg[y];
            }

            // XOR Vx, Vy
            (8, _, _, 3) => {
                self.v_reg[x] ^= self.v_reg[y];
            }

            // ADD Vx, Vy
            (8, _, _, 4) => {
                let (new_vx, carry) = self.v_reg[x].overflowing_add(self.v_reg[y]);

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = carry as u8;
            }

            // SUB Vx, Vy
            (8, _, _, 5) => {
                let (new_vx, borrow) = self.v_reg[x].overflowing_sub(self.v_reg[y]);

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = !borrow as u8;
            }

            // SHR Vx {, Vy}
            (8, _, _, 6) => {
                let lsb = self.v_reg[x] & 1;

                self.v_reg[x] >>= 1;
                self.v_reg[0xF] = lsb;
            }

            // SUBN Vx, Vy
            (8, _, _, 7) => {
                let (new_vx, borrow) = self.v_reg[y].overflowing_sub(self.v_reg[x]);

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = !borrow as u8;
            }

            // SHL Vx {, Vy}
            (8, _, _, 0xE) => {
                let msb = (self.v_reg[x] >> 7) & 1;

                self.v_reg[x] <<= 1;
                self.v_reg[0xF] = msb;
            }

            // SNE Vx, Vy
            (9, _, _, 0) => {
                if self.v_reg[x] != self.v_reg[y] {
                    self.pc += 2;
                }
            }

            // LD I, addr
            (0xA, _, _, _) => {
                self.i_reg = nnn;
            }

            // JP V0, addr
            (0xB, _, _, _) => {
                self.pc = (self.v_reg[0] as u16) + nnn;
            }

            // RND Vx, byte
            (0xC, _, _, _) => {
                let rnd: u8 = random();

                self.v_reg[x] = rnd & nn;
            }

            // DRAW
            (0xD, _, _, _) => {
                let x_coord = self.v_reg[x] as u16;
                let y_coord = self.v_reg[y] as u16;

                let num_rows = digit4 as u16;
                let mut flipped: bool = false;

                // Iterate over each row of our sprite
                for y_line in 0..num_rows {
                    let addr = self.i_reg + y_line as u16;
                    let pixels = self.ram[addr as usize];

                    for x_line in 0..8 {
                        // Use a mask to fetch current pixel's bit.
                        // Only flip if it's a 1
                        if (pixels & (0b1000_000 >> x_line)) != 0 {
                            let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                            let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;

                            let idx = x + SCREEN_WIDTH * y;

                            flipped |= self.screen[idx];

                            self.screen[idx] ^= true;
                        }
                    }
                }

                // Populate VF register with flipped condition
                self.v_reg[0xF] = flipped as u8;
            }

            // SKP Vx
            (0xE, _, 9, 0xE) => {
                let key_down = self.keys[self.v_reg[x] as usize];

                if key_down {
                    self.pc += 2;
                }
            }

            // SKNP Vx
            (0xE, _, 0xA, 1) => {
                let key_down = self.keys[self.v_reg[x] as usize];

                if !key_down {
                    self.pc += 2;
                }
            }

            // LD Vx, DT
            (0xF, _, 0, 7) => {
                self.v_reg[x] = self.dt;
            }

            // LD Vx, K
            (0xF, _, 0, 0xA) => {
                let mut pressed = false;

                // Loop over pressed keys, wait until key is pressed
                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        self.v_reg[x] = i as u8;

                        pressed = true;

                        break;
                    }
                }

                // Loop until we have a keypress
                if !pressed {
                    self.pc -= 2;
                }
            }

            // LD DT, Vx
            (0xF, _, 1, 5) => {
                self.dt = self.v_reg[x];
            }

            // LD ST, Vx
            (0xF, _, 1, 8) => {
                self.st = self.v_reg[x];
            }

            // ADD I, Vx
            (0xF, _, 1, 0xE) => {
                self.i_reg = self.i_reg.wrapping_add(self.v_reg[x] as u16);
            }

            // LD F, Vx
            (0xF, _, 2, 9) => {
                self.i_reg = (self.v_reg[x] as u16) * 5;
            }

            // LD B, Vx
            (0xF, _, 3, 3) => {
                let vx = self.v_reg[x] as f32;

                let hundreds = (vx / 100.0).floor() as u8;

                let tens = ((vx / 100.0) % 10.0).floor() as u8;

                let ones = (vx % 10.0) as u8;

                self.ram[self.i_reg as usize] = hundreds;
                self.ram[(self.i_reg + 1) as usize] = tens;
                self.ram[(self.i_reg + 2) as usize] = ones;
            }

            // LD [I], Vx
            (0xF, _, 5, 5) => {
                for idx in 0..=x {
                    self.ram[(self.i_reg as usize) + idx] = self.v_reg[idx];
                }
            }

            // LD Vx, [I]
            (0xF, _, 6, 5) => {
                for idx in 0..=x {
                    self.v_reg[idx] = self.ram[(self.i_reg as usize) + idx];
                }
            }

            // Failsafe
            (_, _, _, _) => unimplemented!("Unimplemented opcode: {}", op),
        }
    }

    // Tick Timers
    pub fn tick_timers(&mut self) {
        // Decrement Delay Timer if it has a value
        if self.dt > 0 {
            self.dt -= 1;
        }

        // Decrement Sound Timber if it has a value + Play sound
        if self.st == 1 {
            // Play sound
        }

        self.st -= 1;
    }

    // Adds the given u16 value to the top of the stack
    fn push(&mut self, value: u16) {
        self.stack[self.sp as usize] = value;

        self.sp += 1;
    }

    // Returns the value at the top of the stack
    fn pop(&mut self) -> u16 {
        self.sp -= 1;

        self.stack[self.sp as usize]
    }
}
