pub const USAGE_STRING: &'static str = "
IDOLA PSO Server.

Usage:
    idola [options]
    idola (-h | --help)
    idola --version

Options:
    -c,--config=<config>  Config path.
    -h,--help             This message.
    --version             Print version.

The config path defaults to 'idola.toml'. If no file exists, the program
will immediately exit.

The configuration file describes what services to run in this instance of the
server. There are several kinds of services. The config in
data/default/conf_local.toml is configured to spin up all the required services
for PSOBB to connect and play. See the docs for more details.
";

#[derive(Debug, Clone, RustcDecodable)]
pub struct Args {
    pub arg_config: Option<String>,
    pub flag_version: bool
}

impl Args {
    // pub fn config(&self) -> String {
    //     self.arg_config.clone().unwrap_or("idola_pat.toml".to_string())
    // }
}
