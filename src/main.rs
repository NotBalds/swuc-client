use anyhow::Result;
use clap::ArgMatches;

fn main() -> Result<()> {
    let matches = swuc::cli::build_cli().get_matches();

    match matches.subcommand() {
        Some(("run", sub_m)) => handle_run_command(sub_m),
        _ => unreachable!(),
    }
}

fn handle_run_command(matches: &ArgMatches) -> Result<()> {
    let user_cfg = swuc::config::load_user_config()?;
    let pathin = matches.get_one::<String>("pathin").unwrap();
    let pathout = matches.get_one::<&str>("pathout");

    let packages = swuc::storage::load_package_list(pathin)?;
    if packages.is_empty() {
        anyhow::bail!("No packages found in {}", pathin);
    }

    let interval = matches
        .get_one::<String>("interval")
        .and_then(|i| i.parse::<u64>().ok());

    swuc::update::check_updates_with_interval(&user_cfg, &packages, pathout.map(|v| &**v), interval)
}
