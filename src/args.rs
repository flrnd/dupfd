use std::{env, path::Path};

use crate::Config;

pub fn parse() -> Result<Config, String> {
    let args = env::args().skip(1);
    let mut config = Config::default();

    for arg in args {
        let path = Path::new(&arg);

        if path.is_dir() {
            config.inputs.push(path.to_path_buf());
        } else {
            let err = format!("'{}' is not a valid directory path", arg);
            return Err(err);
        }
    }

    Ok(config)
}
