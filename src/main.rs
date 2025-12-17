use ptt_cli::ui::*;
use anyhow::{Result};
use ptt_cli::models::Config;

fn main() -> Result<()>{

    let t_name = "data.json";
    let p_name = "projects.json";
    let mut config = Config::build(t_name, p_name)?;
    
    if let Err(e) = run(&mut config) {
        eprintln!("{e:?}");
        std::process::exit(1);        
    }

    Ok(())
}







