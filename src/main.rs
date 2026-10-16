use riscv_rv32i_interpreter::loader::load;
use riscv_rv32i_interpreter::cpu::{Cpu, StepResult};
use std::io::Write;

const MEMORY_SIZE: usize = 1 << 20;

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: riscv-rv32i-interpreter <program.bin>");
        std::process::exit(2);
    });

    let program = std::fs::read(&path).unwrap_or_else(|e| {
        eprintln!("cannot read {path}: {e}");
        std::process::exit(2);
    });

    let mut cpu = Cpu::new(MEMORY_SIZE);

    if let Err(e) = load(&mut cpu, &program, 0) {
        eprintln!("cannot load program: {e:?}");
        std::process::exit(2);
    }

    let result = loop {
        match cpu.step() {
            Ok(StepResult::Continue) => {}
            other => break other,
        }
    };

    std::io::stdout().write_all(&cpu.output).unwrap();

    match result {
        Ok(StepResult::Exit(code)) => std::process::exit(code as i32),
        Err(e) => {
            eprintln!("error at pc {:#010x}: {e:?}", cpu.pc);
            std::process::exit(1);
        }
        Ok(StepResult::Continue) => unreachable!(),
    }
}
