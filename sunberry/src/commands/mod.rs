use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Core {
    /// the current decompiler
    Old,
    /// the rewrite: refuses what it can't do exactly instead of guessing, and then falls
    /// back to the old core's output, marked as such
    Next,
}

mod decompile;
pub use decompile::*;
mod serve;
pub use serve::*;

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Decompile a single bytecode file, either raw or base64 encoded
    Decompile {
        /// The path to the bytecode
        #[arg(short, long)]
        input: PathBuf,

        /// The path to output the bytecode
        #[arg(short, long)]
        output: PathBuf,

        /// If the supplied bytecode is Luau, what encode key to use (shuffles instructions)
        #[arg(short, long, default_value_t = default_encode_key())]
        encode_key: u8,

        /// Indicate the bytecode is Lua 5.1, assumes Luau by default
        #[arg(short, long, default_value_t = false)]
        lua51: bool,

        /// The name of the script, used to name the table a module returns. Defaults to
        /// the input file's name
        #[arg(short, long)]
        name: Option<String>,

        /// Which decompiler core to use for Luau
        #[arg(long, value_enum, default_value_t = Core::Old)]
        core: Core,
    },

    /// Disassemble Luau bytecode (raw or base64) with the new core's reader
    Disasm {
        #[arg(short, long)]
        input: PathBuf,

        #[arg(short, long, default_value_t = default_encode_key())]
        encode_key: u8,
    },

    /// Start a simple web server to handle decompiling.
    Serve {
        /// The port the web server should listen on
        #[arg(short, long, default_value_t = 3000)]
        port: u16,

        /// Whether to allow decompiling Luau bytecode
        #[arg(short, long, default_value_t = true)]
        luau: bool,

        /// Whether to allow decompiling Lua 5.1 bytecode
        #[arg(short, long, default_value_t = false)]
        lua51: bool,

        /// Which decompiler core to use for Luau when a request doesn't say (`?core=`)
        #[arg(long, value_enum, default_value_t = Core::Old)]
        core: Core,
    },
}

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}
