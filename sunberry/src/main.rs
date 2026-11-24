use clap::Parser;

use crate::commands::{Cli, decompile, serve};

mod commands;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() -> Result<(), std::io::Error> {
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
            strict,
        } => decompile(&input, &output, encode_key, lua51, name, core, strict)?,
        commands::Commands::Disasm { input, encode_key } => commands::disasm(&input, encode_key)?,
        // only the server needs the async runtime and the logger: starting them for a
        // one-shot decompile costs more CPU than the decompile itself on a small script
        commands::Commands::Serve { port, luau, lua51, core } => {
            let subscriber = tracing_subscriber::fmt()
                .compact()
                .with_file(true)
                .with_line_number(true)
                .with_thread_ids(true)
                .with_target(false)
                .finish();
            tracing::subscriber::set_global_default(subscriber)
                .expect("failed to set global tracing subscriber");
            tokio::runtime::Runtime::new()?.block_on(serve(port, luau, lua51, core))?
        }
    }

    // Done
    Ok(())
}
