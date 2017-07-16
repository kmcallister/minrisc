extern crate minrisc;

use std::env;
use std::io;
use minrisc::emu::{Machine, StepOutcome};
use minrisc::decode::{self, Reg};

// Program to compute Fibonacci numbers
static FIB: &[u32] = &[
    0x02050663,  // beqz    a0,2c <.L4>
    0xfff50793,  // addi    a5,a0,-1
    0x02078663,  // beqz    a5,34 <.L5>
    0x00100713,  // li      a4,1
    0x00000693,  // li      a3,0
    0x00e68533,  // add     a0,a3,a4
    0xfff78793,  // addi    a5,a5,-1
    0x00070693,  // mv      a3,a4
    0x00050713,  // mv      a4,a0
    0xfe0798e3,  // bnez    a5,14 <.L3>
    0x00000073,  // ecall
    0x00000513,  // li      a0,0
    0x00000073,  // ecall
    0x00100513,  // li      a0,1
    0x00000073,  // ecall
];

fn main() {
    let arg: u32 = match env::args().nth(1) {
        None => panic!("Usage: ./fib <N>"),
        Some(s) => s.parse().unwrap(),
    };

    let mut machine = Machine::with_memory(64*1024);
    machine.set_reg(Reg::a0(), arg);

    // load program
    for (i, &word) in FIB.iter().enumerate() {
        machine.store32(4*i as u32, word).unwrap();
    }

    loop {
        machine.dump(&mut io::stdout());
        print!("\n");

        let current_inst_bits = machine.load32(machine.pc).unwrap();
        print!("{:?}\n\n", decode::decode(current_inst_bits).unwrap());

        let res = machine.step();
        match res {
            e @ Err(_) => { e.unwrap(); }
            Ok(StepOutcome::Syscall) => {
                println!("Done! Result = {}", machine.get_reg(Reg::a0()));
                break;
            }
            _ => (),
        }
    }
}
