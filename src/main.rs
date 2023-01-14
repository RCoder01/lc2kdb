use std::{fs::File, io::{BufReader, BufRead, Write}, iter, str::FromStr};

mod cpu;

#[derive(Debug)]
enum Error {
    FileNotFound,
    NotEnoughArguments,
    StdoutError,
    StdinError,
}

const HELP_MESSAGE: &'static str = r#"\
h|help           -> show this help message
s|step `n`       -> step program forward `n` steps (default: 1)
r|regs           -> show current register values
m|mem `addr` `n` -> read `n` bits starting from address `addr`
p|pc             -> display current program counter
q|quit           -> close debugger"#;

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
        if let Err(_) = reader.read_line(&mut line) {
            None
        } else {
            line.trim_end().parse::<isize>().ok().map(|n| n as u32)
        }
    };

    let mut cpu = cpu::CPU::new(iter::from_fn(read));

    let mut line = String::new();
    loop {
        get_input(&mut line)?;
        let args = line.trim().split_whitespace();
        match process_repl_input(args, &mut cpu) {
            Err(_) => println!("Unrecognized command. Use `help` for usage."),
            Ok(quit) => {
                if quit {break;};
            },
        }
        line.clear();
    };
    Ok(())
}


fn get_input(str: &mut String) -> Result<(), Error> {
    print!(">>> ");
    std::io::stdout().flush().map_err(|_| Error::StdoutError)?;
    std::io::stdin().read_line(str).map_err(|_| Error::StdinError)?;
    Ok(())
}


struct UnrecognizedCommandError;

fn process_repl_input<'a, T: Iterator<Item = &'a str>>(mut args: T, cpu: &mut cpu::CPU) -> Result<bool, UnrecognizedCommandError> {
    let op = args.next().ok_or(UnrecognizedCommandError)?;
    match op {
        "help" | "h" => println!("{}", HELP_MESSAGE),
        "step" | "s" => {
            let count = match parse_from_arg::<usize>(args.next()) {
                Ok(count) => count,
                Err(_) => 1,
            };
            if cpu.step_n(count) {
                println!("Program has halted");
            }
        },
        "regs" | "r" => cpu.print_registers(),
        "mem" | "m" => {
            let addr = parse_from_arg::<u32>(args.next())?;
            let count = parse_from_arg::<u32>(args.next())?;
            cpu.print_memory(addr, count);
        },
        "pc" | "p" => {
            cpu.print_program_counter();
        },
        "quit" | "q" => {
            return Ok(true);
        },
        _ => {
            return Err(UnrecognizedCommandError);
        }
    };
    Ok(false)
}

fn parse_from_arg<'a, T: FromStr>(arg: Option<&'a str>) -> Result<T, UnrecognizedCommandError> {
    arg.ok_or(UnrecognizedCommandError)?.parse::<T>().map_err(|_| UnrecognizedCommandError)
}
