//! `hal spend`: this week's cost against the budget, for scripts.
use super::CliError;
use hal_lib::spend;

pub(crate) fn spend(args: &[String]) -> Result<(), CliError> {
    let mut force = false;
    let mut reset = false;
    for arg in args {
        match arg.as_str() {
            "--refresh" => force = true,
            "--reset" => reset = true,
            other => return Err(CliError::Usage(format!("unknown argument: {other}"))),
        }
    }
    let spend = if reset {
        spend::reset()
    } else if force {
        spend::refresh()
    } else {
        spend::read()
    };
    super::print_json(&spend.map_err(CliError::Other)?.report());
    Ok(())
}
