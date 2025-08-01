mod parser;
mod resolver;
mod generator;
mod build;
mod cli;
mod ui;
mod platform;
mod commands;
mod shell_runtime;

use anyhow::Result;
use clap::Parser;
use log::info;

fn main() -> Result<()> {
    env_logger::init();
    
    let args = cli::Args::parse();
    info!("Starting cassh2rs v{}", env!("CARGO_PKG_VERSION"));
    
    cli::run(args)
}