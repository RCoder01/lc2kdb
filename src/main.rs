use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
    iter,
    str::FromStr,
};

mod cpu;

#[derive(Debug)]
enum Error {
    FileNotFound,
    NotEnoughArguments,
    Stdout,
    Stdin,
}

const HELP_MESSAGE: &str = r#"h|help           -> show this help message
s|step `n`       -> step program forward `n` steps (default: 1)
r|regs           -> show current register values
m|mem `addr` `n` -> read `n` (default: 1) bits starting from address `addr` (default: 0)
p|pc             -> display current program counter
i|ins `n`        -> print the `n` (default: 1) memory addresses after pc as instructions
q|quit           -> close debugger
c|count          -> print number of clock cycles executed"#;

fn main() -> Result<(), Error> {
    let args = std::env::args();
    let args = args.collect::<Vec<_>>();
    let mut reader = if let Some(ifname) = args.get(1) {
        BufReader::new(File::open(ifname).map_err(|_| Error::FileNotFound)?)
    } else {
        return Err(Error::NotEnoughArguments);
    };

    let mut line = String::new();
    let read = || {
        line.clear();
        if reader.read_line(&mut line).is_err() {
            None
        } else {
            line.trim_end().parse::<i64>().ok().map(|n| n as u32)
        }
    };

    let mut cpu = cpu::CPU::new(iter::from_fn(read));

    let mut line = String::new();
    loop {
        get_input(&mut line)?;
        let args = line.split_whitespace();
        match process_repl_input(args, &mut cpu) {
            Err(_) => println!("Unrecognized command. Use `help` for usage."),
            Ok(quit) => {
                if quit {
                    break;
                };
            }
        }
        line.clear();
    }
    Ok(())
}

fn get_input(str: &mut String) -> Result<(), Error> {
    print!(">>> ");
    std::io::stdout().flush().map_err(|_| Error::Stdout)?;
    std::io::stdin().read_line(str).map_err(|_| Error::Stdin)?;
    Ok(())
}

struct UnrecognizedCommandError;

fn process_repl_input<'a, T: Iterator<Item = &'a str>>(
    mut args: T,
    cpu: &mut cpu::CPU,
) -> Result<bool, UnrecognizedCommandError> {
    let op = args.next().ok_or(UnrecognizedCommandError)?;
    match op {
        "help" | "h" => println!("{}", HELP_MESSAGE),
        "step" | "s" => {
            let count = parse_from_arg::<usize>(args.next()).unwrap_or(1);
            if cpu.step_n(count) {
                println!("Program has halted");
            }
        }
        "regs" | "r" => cpu.print_registers(),
        "mem" | "m" => {
            let addr = parse_from_arg::<u32>(args.next()).unwrap_or(0);
            let count = parse_from_arg::<u32>(args.next()).unwrap_or(1);
            cpu.print_memory(addr, count);
        }
        "pc" | "p" => {
            cpu.print_program_counter();
        }
        "ins" | "i" => {
            let count = parse_from_arg::<usize>(args.next()).unwrap_or(1);
            cpu.print_instruction(count);
        }
        "quit" | "q" => {
            return Ok(true);
        }
        "count" | "c" => {
            println!("{}", cpu.get_instruction_count());
        }
        _ => {
            return Err(UnrecognizedCommandError);
        }
    };
    Ok(false)
}

fn parse_from_arg<T: FromStr>(arg: Option<&'_ str>) -> Result<T, UnrecognizedCommandError> {
    arg.ok_or(UnrecognizedCommandError)?
        .parse::<T>()
        .map_err(|_| UnrecognizedCommandError)
}
