use clap::Parser;

/// CLI Args via parser
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Set RPC URL
    #[clap(long, default_value = "http://127.0.0.1:8545")]
    rpc: String
}


/// Global settings
#[derive(Debug, Clone)]
pub struct Settings {
    pub rpc_url: String
}

impl Default for Settings {
    /// Default settings for the tool
    fn default() -> Self {
        let args = Args::parse();

        Self {
            rpc_url: args.rpc
        }
    }
}
impl std::fmt::Display for Settings {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "Simulator Settings")?;
        writeln!(f, "-> rpc: {:?}", self.rpc_url)
    }
}