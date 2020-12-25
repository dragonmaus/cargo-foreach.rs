use getopt::Opt;
use std::{
    fs::{self, DirEntry},
    io::{self, ErrorKind},
    path::PathBuf,
    process::{Command, Stdio},
};

program::main!("cargo-foreach");

fn usage_line() -> String {
    format!(
        "Usage: {} [-h] [-qv] [-C DIR] command [args...]",
        program::name("cargo-foreach")
    )
}

fn print_usage() {
    println!("{}", usage_line());
    println!("  -C DIR  switch to DIR before starting");
    println!("  -q      suppress command error output");
    println!("  -v      print directory names as they are processed");
    println!();
    println!("  -h      display this help");
}

fn program() -> program::Result {
    let mut args = program::args();
    // cargo passes all of its arguments unchanged to subcommands
    if args[1] == "foreach" {
        args.remove(1);
    }
    let mut opts = getopt::Parser::new(&args, "C:hqv");
    let mut base = PathBuf::from(".");
    let mut quiet = false;
    let mut verbose = false;

    loop {
        match opts.next().transpose()? {
            None => break,
            Some(opt) => match opt {
                Opt('C', Some(p)) => base = PathBuf::from(p),
                Opt('q', None) => quiet = true,
                Opt('v', None) => verbose = true,
                Opt('h', None) => {
                    print_usage();
                    return Ok(0);
                }
                _ => unreachable!(),
            },
        }
    }

    if !base.is_dir() {
        return program::error(ErrorKind::NotFound, &format!("{:?}: not a directory", base));
    }

    let mut args = args.split_off(opts.index());
    if args.is_empty() {
        eprintln!("{}", usage_line());
        return Ok(1);
    }
    let cmd = args.remove(0);

    let mut entries = fs::read_dir(base)?.collect::<io::Result<Vec<DirEntry>>>()?;
    entries.sort_unstable_by_key(|a| a.file_name());
    for entry in entries {
        if !entry.path().is_dir() || !entry.path().join("Cargo.toml").exists() {
            continue;
        }

        if verbose {
            println!(">> {}", entry.file_name().to_string_lossy());
        }

        let status = Command::new(cmd.to_string())
            .args(&args)
            .current_dir(entry.path())
            .stderr(if quiet {
                Stdio::null()
            } else {
                Stdio::inherit()
            })
            .status()?;
        if !quiet && !status.success() {
            match status.code() {
                None => eprintln!("process didn't exit successfully (terminated by signal)"),
                Some(code) => eprintln!("process didn't exit successfully (exit code: {})", code),
            };
        }
    }

    Ok(0)
}
