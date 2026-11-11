//! `hal keys`: every place and action `[keymap]` can name, with the keys in
//! force.
use super::CliError;
use hal_lib::keymap::{self, Scheme};

pub(crate) fn keys(args: &[String]) -> Result<(), CliError> {
    let scheme = match args {
        [] => hal_lib::config::get().ui.keys,
        [name] => Scheme::ALL
            .into_iter()
            .find(|s| s.config_value() == name)
            .ok_or_else(|| CliError::Usage(format!("unknown scheme: {name} (ide or vim)")))?,
        _ => return Err(CliError::Usage("usage: hal keys [ide|vim]".into())),
    };
    print!("{}", keymap::dump(scheme));
    for problem in keymap::problems() {
        eprintln!("warning: [keymap] {}", problem);
    }
    Ok(())
}
