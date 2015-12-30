pub const USAGE_STRING: &'static str = "
IDOLA PSO Patch server.

Usage:
    idola_pat [options]
    idola_pat (-h | --help)
    idola_pat --version

Options:
    -c,--config=<config>  Config path.
    -h,--help             This message.
    --version             Print version.

The config path defaults to 'idola_pat.toml'. If no file exists, the program
will immediately exit.
";

#[derive(Debug, Clone, RustcDecodable)]
pub struct Args {
    arg_config: Option<String>,
    flag_version: bool
}

impl Args {
    // pub fn config(&self) -> String {
    //     self.arg_config.clone().unwrap_or("idola_pat.toml".to_string())
    // }
}
