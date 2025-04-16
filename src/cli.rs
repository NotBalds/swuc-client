use clap::{Arg, Command};

pub fn build_cli() -> Command {
    Command::new("swuc")
        .version("0.1.0")
        .about("Secure Software Updates Checker")
        .subcommand_required(true)
        .subcommand(
            Command::new("run")
                .about("Start update checks")
                .arg(
                    Arg::new("pathin")
                        .short('i')
                        .long("input")
                        .required(true)
                        .help("Path to package list file"),
                )
                .arg(
                    Arg::new("pathout")
                        .short('o')
                        .long("output")
                        .required(false)
                        .help("Path to save human-readable report"),
                )
                .arg(
                    Arg::new("interval")
                        .short('t')
                        .long("interval")
                        .help("Check interval in hours"),
                ),
        )
}
