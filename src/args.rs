pub const USAGE_STRING: &'static str = "
IDOLA PSO Server.

Usage:
    idola [options]
    idola (-h | --help)
    idola --version

Options:
    --config=<config>        Config path [default: idola.toml].
    -h,--help                This message.
    --version                Print version.

The config path defaults to 'idola.toml'. If no file exists, the program
will immediately exit.

The configuration file describes what services to run in this instance of the
server. There are several kinds of services. The config in
data/default/conf_local.toml is configured to spin up all the required services
for PSOBB to connect and play. See the docs for more details.
";

#[derive(Debug, Clone, RustcDecodable)]
pub struct Args {
    pub flag_config: String,
    pub flag_version: bool
}
