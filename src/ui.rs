use std::vec;

use crate::models::*;
use crate::utils::*;
//use crate::storage::*;
use inquire::{Confirm, Select};
use anyhow::Result;
// #DELETE? Lines probably due to deletion

/// Main function of the code. Gets called by main.rs
pub fn run(config: &mut Config) -> Result<()> {

    if config.project_records.is_empty() {
        let confirm = Confirm::new("There are currently no projects! Would you like to add one first? (Y/n): ").prompt();

        if matches!(confirm, Ok(true)) {
            add_project(config)?;
        } else {
            println!("No projects in memory! It is advised to add at least one first!");
        }
    }
    
    main_menu(config)?;
    Ok(())
}

/// Main menu of the App. Always where the user starts
pub fn main_menu(config: &mut Config) -> Result<()> {
    // get the menu structure via a vec
    let menu_options = vec!["Log Time", "Projects", "Reports", "Exit"];
    loop {
        if let Ok(ans) = Select::new("What do you want to do?", menu_options.clone()).prompt()  {
            
            match ans {
                "Log Time" => log_time_menu(config)?,
                "Projects" => projects_menu(config)?,
                "Reports" => reports_menu()?,
                "Exit" => {
                    println!("Goodbye!");
                    break Ok(());
                },
                &_ => break Ok(()),
            }
        }
    }   
}

/// Menu entry everything time related. Workday or Project Work can be entered here
pub fn log_time_menu(config: &mut Config) -> Result<()> {

    // TODO: Read in Values beforehand

    let log_time_options = vec!["Record Workday", "Record Project Work" ,"Edit Workday Record", "Delete Workday","Back", "Exit"];

    loop {

        if let Ok(ans) = Select::new("Log Time Menu", log_time_options.clone()).prompt() {
            match ans {
                "Record Workday" => record_time_record(config)?, // If Record gets overwritten, loses all entries for project_entries up so far
                "Record Project Work" => record_project_work(config)?,
                "Edit Workday Record" => edit_workday_record(config)?,
                "Delete Workday" => {
                    let (selected_date, _) = match choose_date(config, "Which record would you like to delete?") {
                        Ok(Some(value)) => value,
                        Ok(None) => {
                            println!("Operation cancelled. Returning to main...");
                            return Ok(());
                        },
                        Err(e) => return Err(e),
                    };
                    config.delete_time_record(selected_date)?;
                },
                "Back" => break Ok(()),
                "Exit" => {
                    println!("Goodbye");
                    std::process::exit(0)},
                &_ => todo!()
            }
        }
    }
}


/// Menu entry for everything project related. 
pub fn projects_menu(config: &mut Config) -> Result<()>{
    println!("Yay, let's work with som projects");

    loop {
        let options = vec!["List Projects", "Add Project", "Delete Project", "Back", "Exit"];
        match Select::new("Project Menu", options).prompt() {
            Ok("List Projects") => list_projects(config)?,
            Ok("Add Project") => add_project(config)?,            
            Ok("Delete Project") => {
                
                if config.project_records.is_empty() {
                    println!("No projects to delete.");
                    continue;
                }

                if let Ok(to_delete) = Select::new("Which project do you want to delete?", 
                config.project_records.iter().map(|p| p.code.clone()).collect()).prompt(){
                    config.delete_project(to_delete)?;
                }
            }

            Ok("Back") => break Ok(()),

            Ok("Exit") => {
                println!("Goodbye");
                std::process::exit(0);
            },
            _ => continue,
        }
    }


}

/*
pub fn view_edit_menu(config:&mut Config) -> Result<()>{
    loop {
        let options = vec!["Edit Workday", "Edit Project Entry", "Back", "Break"];
        match Select::new("Vies / Edit Menu", options).prompt() {
            Ok("Edit Workday") => edit_workday_record(config)?, 
            Ok("Back") => break Ok(()),
            Ok("Exit") => {
                println!("Goodbye");
                std::process::exit(0);
            },
            _=> continue,
        }        
    }

} */


pub fn reports_menu() -> Result<()>{
    println!("TBD! Print some reports");
    return Ok(());
}