use anyhow::{anyhow, Result};
use std::{
    io::{Read, Write},
    path::Path,
};
use sysinfo::{Pid, System, SystemExt};

use clap::Parser;
use frontend::Args;

mod disks;
mod frontend;
mod install;
mod network;
mod parser;

const LOCK: &str = "/run/lock/aoscdk.lock";

fn main() {
    if let Err(e) = create_lock() {
        eprintln!("AOSC OS Installer failed to obtain the instance lock: {}", e);
        std::process::exit(1);
    }
    if let Err(e) = execute() {
        eprintln!("{}", e);
        remove_lock().ok();
        std::process::exit(1);
    }
    remove_lock().ok();
    std::process::exit(0);
}

fn execute() -> Result<()> {
    let args = std::env::args();
    if args.len() < 2 {
        frontend::tui_main();
    } else {
        let args = Args::parse();
        frontend::execute(args)?;
    }

    Ok(())
}

fn create_lock() -> Result<()> {
    let lock = Path::new(LOCK);
    if lock.is_file() {
        let mut lock_file = std::fs::File::open(lock)?;
        let mut buf = String::new();
        lock_file.read_to_string(&mut buf)?;
        let old_pid = buf
            .parse::<i32>()
            .map_err(|_| anyhow!("Invalid or corrupted lock file!"))?;
        let s = System::new_all();
        if s.process(Pid::from(old_pid)).is_some() {
            return Err(anyhow!(
                "Another instance of AOSC OS Installer (pid: {}) is still running!",
                old_pid
            ));
        } else {
            remove_lock()?;
        }
    }
    let mut lock_file = std::fs::File::create(lock)?;
    let pid = std::process::id().to_string();
    lock_file.write_all(pid.as_bytes())?;

    Ok(())
}

fn remove_lock() -> Result<()> {
    std::fs::remove_file(LOCK)?;

    Ok(())
}
