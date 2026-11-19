use clap::Parser;

use crate::commands::{Cli, decompile, serve};

mod commands;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Setup the logger
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("failed to set global tracing subscriber");

    // Run the requested command
    let cli = Cli::parse();
    match cli.command {
        commands::Commands::Decompile {
            input,
            output,
            encode_key,
            lua51,
            name,
            core,
        } => decompile(&input, &output, encode_key, lua51, name, core)?,
        commands::Commands::Disasm { input, encode_key } => commands::disasm(&input, encode_key)?,
        commands::Commands::Serve { port, luau, lua51 } => serve(port, luau, lua51).await?,
    }

    // Done
    Ok(())
}
