use clap::Parser;
use ethers::types::H160;
use std::str::FromStr;

/// CLI Args via parser
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Set RPC URL
    #[clap(long, default_value = "http://127.0.0.1:8545")]
    rpc: String,
    /// Router
    #[clap(long, default_value = "0x7a250d5630b4cf539739df2c5dacb4c659f2488d")]
    router: String,
    /// Factory
    #[clap(long, default_value = "0x5c69bee701ef814a2b6a3edd4b1652cb9cc5aa6f")]
    factory: String,
}

/// Global settings
#[derive(Debug, Clone)]
pub struct Settings {
    pub rpc_url: String,
    pub router: H160,
    pub factory: H160
}

impl Default for Settings {
    /// Default settings for the tool
    fn default() -> Self {
        let args = Args::parse();

        Self {
            rpc_url: args.rpc,
            router: H160::from_str(&args.router).unwrap(),
            factory: H160::from_str(&args.factory).unwrap(),
        }
    }
}
impl std::fmt::Display for Settings {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "Simulator Settings")?;
        let _ = writeln!(f, "-> rpc:        {:?}", self.rpc_url);
        let _ = writeln!(f, "-> factory:    {:?}", self.factory);
        writeln!(f, "-> router:     {:?}", self.router)
    }
}