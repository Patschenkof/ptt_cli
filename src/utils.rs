use std::collections::HashMap;
use std::vec;

use chrono::{Datelike, Local, NaiveDate, NaiveTime, Weekday};
use inquire::InquireError;
use inquire::{CustomType, DateSelect, Select, Text, validator::Validation, Confirm};
use crate::models::{TimeRecord, Project, ProjectEntry};
use crate::models::*;
use anyhow::{Context, Result, anyhow};


/// Error handling when user hits esc:
fn prompt_input(prompt: &str) -> Result<Option<String>>{
    match Text::new(prompt).prompt() {
        Ok(s) => Ok(Some(s)),
        Err(InquireError::OperationCanceled) | Err(InquireError::OperationInterrupted) => Ok(None),
        Err(e)=> Err(e.into())
    }
}

fn prompt_select<T, F>(prompt: &str, options: Vec<&str>, map_fn : F) -> Result<Option<T>>
where 
    F: Fn(&str) -> T,
    {
        match Select::new(prompt, options).prompt() {
            Ok(selection) => Ok(Some(map_fn(selection))),
            Err(InquireError::OperationCanceled) | Err(InquireError::OperationInterrupted) => Ok(None),
            Err(e) => Err(e.into())
        }
    } 

fn prompt_parse<T,F>(prompt: &str, mut parse_fn: F) -> Result<Option<T>>
where 
    F: FnMut(&str) -> Result<T, String>,
    {
        loop {
            match prompt_input(prompt).map_err(|e| anyhow!(e))? {
                Some(input) => match parse_fn(&input) {
                    Ok(value)=> return Ok(Some(value)),
                    Err(msg)=> {
                        println!("{}", msg);
                        continue;
                    }
                },
                None => return Ok(None)
                
            }
        }
    }

/// User can set his workday here. Will open up a calender for the user to select date.
pub fn record_time_record(config: &mut Config) -> Result<()> {

    let date = match DateSelect::new("Enter Date of Work:")
        .with_starting_date(NaiveDate::from(Local::now().date_naive()))
        .with_min_date(NaiveDate::from_ymd_opt(2025, 01, 01).unwrap())
        .with_max_date(NaiveDate::from_ymd_opt(2027, 12, 31).unwrap())
        .with_week_start(Weekday::Mon)
        .with_help_message("Select Day from the calendar")
        .prompt() {
            Ok(date)=> date,
            Err(InquireError::OperationCanceled) | Err(InquireError::OperationInterrupted) => {
                println!("Operation cancelled. Returning to menu...");
                return Ok(());
            },
            Err(e) => return Err(e.into()),
        };

    
    // Check for existing record
    if let Some(existing) = config.time_records.iter_mut().find(|r| r.date == date){
        let confirm = match Confirm::new(&format!("A record for {} already exist. Do you want to override? (Y/n)", date))
            .prompt() {
                Ok(confirm) => confirm,
                Err(InquireError::OperationCanceled) | Err(InquireError::OperationInterrupted) => {
                    println!("Operation cancelled. Returning to menu...");
                    return Ok(());
                },
                Err(e)=> return Err(e.into()),
            };

        if matches!(confirm, true){
                // Get Information to construct struc
                let start_time = match record_time("When did you start to work?: ")? {
                    Some(start_time)=> start_time,
                    None =>{
                        println!("Operation cancelled! Returning to menu...");
                        return Ok(());
                    },
                };
                let end_time = match record_time("When did you end your work?: ")?{
                    Some(start_time)=> start_time,
                    None =>{
                        println!("Operation cancelled! Returning to menu...");
                        return Ok(());
                    },               
                };
                let pause_minutes = match pause_minutes()? {
                    Some(pause) => pause,
                    None => {
                        println!("Operation Cancelled. Returning to menu...");
                        return Ok(());
                    },
                };

                // Overwrite existing record
                *existing = TimeRecord { 
                    date,
                    start_time,
                    end_time,
                    pause_minutes,
                    project_entries: vec![] // Overwrites existing project_entries!
                };
                println!("Record Updated");
                println!("You have worked {} hours today", existing.get_net_hours());
                config.save()?;
                return Ok(());
        } else {
            println!("Record not Changed!");
            return Ok(());
        }
    } else {
        // Get Information to construct struc
        let start_time = match record_time("When did you start to work?: ")? {
            Some(start_time)=> start_time,
            None =>{
                println!("Operation cancelled! Returning to menu...");
                return Ok(());
            },
        };
        let end_time = match record_time("When did you end your work?: ")?{
            Some(start_time)=> start_time,
            None =>{
                println!("Operation cancelled! Returning to menu...");
                return Ok(());
            },                 
        };
        let pause_minutes = match pause_minutes()? {
            Some(pause) => pause,
            None => {
                println!("Operation Cancelled. Returning to menu...");
                return Ok(());
            },
        };

        let new_record = TimeRecord{
            date,
            start_time,
            end_time,
            pause_minutes,
            project_entries: vec![]
        };

        config.add_time_record(&new_record)?;
        println!("You have worked {} hours today", new_record.get_net_hours());
        return Ok(());
    }

}

/// Function to record a time windows. Will return NaiveTime.
pub fn record_time(prompt: &str) -> Result<Option<NaiveTime>> {

    prompt_parse(prompt, |s| {
        NaiveTime::parse_from_str(s, "%H:%M")
            .map_err(|_| "Please put in a valid time like 08:00".to_string())
    })

}

/// Let's the user assign how long his pause was. Allows custom values. 
pub fn pause_minutes() -> Result<Option<f64>> {

    let pause_options = vec!["0.5", "0.75", "1", "Custom"];
    let prompt = "How long was your pause today?";

    let pause_t = Select::new(prompt, pause_options).prompt_skippable()?;

    match pause_t {
        Some("0.5") => return Ok(Some(0.5)),
        Some("0.75") => return Ok(Some(0.75)),
        Some("1") => return Ok(Some(1.0)),
        Some("Custom") => {
            let pause_custom = CustomType::<f64>::new("Please enter a number (time quarter)")
                .with_help_message("Type something like 0.5, 0.75, 1 etc.")
                .with_error_message("Please type in a valid number! (0.5, 0.75, 1.0)")
                .with_validator(|input: &f64|{
                    if (input * 4.0).fract() == 0.0 {
                        Ok(Validation::Valid)
                    } else {
                        Ok(Validation::Invalid(
                            "Pause must be in 0.25 hour increments".into()
                        ))
                    }
                }).prompt_skippable()?;

            match pause_custom {
                Some(pause) =>{
                    return Ok(Some(((pause / 60.0) * 4.0).round() /4.0));
                },
                None => return Ok(None),
            }
        },
        None => return Ok(None),
        Some(_) => unreachable!(),

    };
    
}

/// Enter a time frame and an activity on a given workday for a given project
pub fn record_project_work(config: &mut Config) -> Result<()>{

    loop {

        if config.project_records.is_empty() {
            println!("Currently no projects! Please add one first.");
            return Ok(());
        }

        let (time_record_ans, _) = match choose_date(config, "For what date would you like to enter an activity?") {
        Ok(Some(value)) => value,
            Ok(None) => {
                println!("Operation Cancelled. Returning to main...");
                return Ok(());
            },
            Err(e) => return Err(e)
        };
        
        let proj_ans = match choose_project(&config.project_records, "For what project would you like to record an activity?"){
            Ok(Some(project_entry)) => project_entry,
            Ok(None) => {
                println!("Operation cancelled. Returning to main...");
                return Ok(());
            },
            Err(e) => return Err(e)
        };

        // Get the stored Project struct
        let single_proj = match find_project(&config.project_records, &proj_ans){
            Some(single_proj) => single_proj,
            None => {
                return Err(anyhow!("The Project was not found :( \n Returning to main!"));
            }
        };

        // Retrieve the already assigned hous for the workday
        let assigned_hours = match get_activity_hours(&time_record_ans, &config.time_records) {
            Ok(Some(assigned_hours)) => assigned_hours,
            Ok(None) => {
                println!("Operation cancelled. Returning to main...");
                return Ok(());
            }
            Err(e) => return Err(e.into()),
        };

        // Ask the user for his activities in the poject
        let activity = match Text::new("What did you do?:").with_validator(|input: &str| {
            if input.len() <= 500 {
                Ok(Validation::Valid)
            } else {
                Ok(Validation::Invalid("Entry must be no longer than 500 characters!".into()))
            }
            }).prompt() {

            Ok(activity)=> activity,
            Err(InquireError::OperationCanceled) | Err(InquireError::OperationInterrupted) => {
                println!("Operation cancelled, returning to menu...");
                return Ok(());
            },

            Err(e) => return Err(e.into()),
        };
        // Initialize ProjectEntry
        let new_project_entry = ProjectEntry {
            project_name: single_proj, 
            hours: assigned_hours,
            activity: activity
        };

        config.add_project_entry(time_record_ans, new_project_entry)?;
        break Ok(());
    }

}

/// Function to prompt the user for the remaining time. 
/// Adding the date entry and the project, this function will list possible working hours, so that you do not exceed the days amount of work 
pub fn get_activity_hours(date: &NaiveDate, time_record: &Vec<TimeRecord>) -> Result<Option<f64>> {


    let remaining_hours = time_record
    .iter()
    .find(|r| r.date == *date).map(|r| r.remaining_hours())
    .expect("No Hours found for this date!");

    if remaining_hours == 0.0 {
        println!("No hours left to record :(");
        return Ok(None);
    }

    println!("You have {} hours at your disposal. How much would you like to assign?", remaining_hours);
    println!("Already recorded for this date:\n");
    time_record.iter().find(|r| r.date == *date).map(|e| e.print_already_recorded());

    let assigned_hours = loop {

        let assigned_hours = CustomType::<f64>::new("How many hours would you like to assign to your activity?")
            .with_error_message("Please type in a valid number!")
            .with_help_message("Valid numbers are 0.5, 0.75, 1.0, 2.0, 3.5 etc.")
            .prompt();
        let assigned_hours = match assigned_hours {
            Ok(assigned_hours) => assigned_hours,
            Err(InquireError::OperationCanceled) | Err(InquireError::OperationInterrupted) => {
                return Ok(None);
            },
            Err(e) => return Err(e.into()),
        };

        if assigned_hours == 0.0 {
            println!("Please assigne more than zero!");
            continue;
        } else if assigned_hours <= remaining_hours {
            break assigned_hours;
        } else {
            println!("You have assigned to much time. Please try again");
            continue;
        }; 
        
    };
    
    return Ok(Some(assigned_hours));



}


/// Asks the user for the Project information. Calls Config.add_project() to add data to permanent storage
pub fn add_project(config: &mut Config)-> Result<()>{

    //let mut projects: Vec<Project> = load_data("projects.json");

    loop {

        let code = Text::new("Enter a project code: ").with_validator(|input: &str| {
            if input.len() <= 5 {
                Ok(Validation::Valid)
            } else {
                Ok(Validation::Invalid("Code can only be 5 characters long!".into()))
            }
        }).prompt_skippable()?;

        let code  = match code {
            Some(code ) => code,
            None => {
                println!("Operation cancelled. Returning to main...");
                return Ok(());
            }
        };

        let allocation = CustomType::<f64>::new("To what degree have you been allocated to the project?")
            .with_error_message("Please type in a valid value (0.1, 0.2, 0.5 etc.")
            .with_help_message("Type in a percantage like '0.5'")
            .prompt_skippable()?;

        let allocation = match allocation {
            Some(allocation) => allocation,
            None => {
                println!("Operation cancelled. Returning to main...");
                return Ok(());
            }
        };


        if !config.project_records.iter().any(|p| p.code == code) {
            config.add_project(Project { code, allocation })?;
            //projects.push(Project { code, allocation });
            //save_data(&projects, "projects.json");
            //println!("Projct Added");
            break Ok(());
        } else {
            println!("Project already exists");
            continue;
        }
    }



}

/// Searchs for a project code in an &[Project]
/// Returns:
///     Option<Project>
pub fn find_project(projects: &[Project], code: &str) -> Option<Project> {
    if code == "INT" {
        return Some(Project{code: String::from("INT"), allocation: 0.0});
    }

    if code == "Vacation" {
        return Some(Project { code: String::from("Vacation"), allocation: 0.0 });
    }

    projects.iter().find(|p| p.code == code).cloned()
}

/// Clears the screen everytime this function is called. Ideally when the user enters a new submenu
pub fn clear_screen() {
    print!("{esc}c", esc = 27 as char);
}

/// Edit the Project Entries in a TimeRecord
pub fn edit_workday_record(config: &mut Config) -> Result<()> {    

    let (selected_date, _) = match choose_date(config, "For what date would you like to edit the entries?") {
        Ok(Some(value)) => value,
        Ok(None) => {
            println!("Operation cancelled. Returning to main...");
            return Ok(());
        },
        Err(e) => return Err(e)
    };

    if let Some(record) = config.time_records.iter_mut().find(|r| r.date == selected_date) {
        /* Could be written as:
            let record = config
                .time_records
                .iter_mut()
                .find(|r| r.date == selected_date)
                .ok_or_else(
                    || println!("An Error Occured: {}", selected_date)
                );  
         */

        if record.project_entries.is_empty() {
            println!("Currently no project records for {}. Please enter a record first!\n", selected_date);
            println!("Returning to main...");
            return Ok(());
        }

         // Print the user, what he has allocated and what he has done in a project
         println!("These are the records for the selected date: {}", selected_date);
         println!("\n");

         for pe in &record.project_entries{
            println!("++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++");
            println!("Project: {}", pe.project_name.code);
            println!("Assigned hours this day: {}", pe.hours);
            println!("Activity: {}", pe.activity);
            println!("++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++");
            println!("\n");
         }

         let vec_pcodes: Vec<String> = record.project_entries
            .iter()
            .map(|p|p.project_name.code.clone())
            .collect();

         let select_pcode = match Select::new("Which Project entry would you like to edit?",
            vec_pcodes).prompt() {
                Ok(value) => value,
                Err(InquireError::OperationCanceled) | Err(InquireError::OperationInterrupted) => {
                    println!("Operation cancelled. Returning to main...");
                    return Ok(());
                },
                Err(e) => return Err(e.into()),
            };

        
        if let Some(project) = record.project_entries
        // s.o.
            .iter_mut()
            .find(|p| p.project_name.code == select_pcode){


            let assigned_hours = CustomType::<f64>::new("How many hours would you like to assign?")
                .with_default(project.hours)
                .with_error_message("Please type in a valid number")
                .with_help_message("Valid format are 0.5, 1.0, 3.5 etc")
                .prompt_skippable()
                .with_context(|| format!("Failed to assign hours for Project: {:#?}", project.project_name.code))?;

            let assigned_hours = match assigned_hours {
                Some(hours) => hours,
                None => {
                    println!("Operation cancelled. Returning to main...");
                    return Ok(())
                }
            };

            let activity = Text::new("What did you do?")
                .with_default(&project.activity)
                .with_validator(|input: &str| {
                    if input.len() <= 500 {
                        Ok(Validation::Valid)
                    } else {
                        Ok(Validation::Invalid("Activity should be no longer than 500 characters!".into()))
                    }
                })
                .prompt_skippable()
                .with_context(|| format!("Failed to assign activity for Project: {:#?}",project.project_name.code))?;

            let activity = match activity {
                Some(activity) => activity,
                None => {
                    println!("Operation cancelled. Returning to main...");
                    return Ok(());
                }
            };

            // Todo: Overwrite Entry with the new information!

            project.hours = assigned_hours;
            project.activity = activity;

            config.save()?;
            return Ok(());
        } else {
        println!("Something wen't wrong! No record found for project: {:#?}", select_pcode);
        Err(anyhow!("Exiting program! Check you code!"))
        }
    } else {
        println!("Something wen't wrong! No record found for date: {:#?}", selected_date);
        Err(anyhow!("Exiting program! Check you code!"))
    }
}

/// Call function to prompt the user a list of the currenty stored workdays
pub fn choose_date(config: &Config, prompt: &str) -> Result<Option<(NaiveDate, TimeRecord)>> {

    // get a mut vec of the naive dates
    let mut time_record_entries: Vec<NaiveDate>= config.time_records.iter().map(|e| e.date.clone()).collect(); 

    // Sort newes to oldest
    time_record_entries.sort_by(|a,b| b.cmp(a));


    let entry = match Select::new(prompt, time_record_entries).prompt() {
        Ok(entry) => entry,
        Err(InquireError::OperationCanceled) | Err(InquireError::OperationInterrupted) => {
            return Ok(None);
        },
        Err(e) => return Err(e.into()),
    };

    if let Some(tr) = config.time_records.iter().find(|r| r.date == entry){

        return Ok(Some((entry, tr.clone())));

    } else {
        return Err(anyhow!("Could not find a time record. Something went wrong"));
    }

}

/// Quite literaly lists all the projects given in a &[Project] type
pub fn list_projects(config: &Config) -> Result<()> {

    
    if config.project_records.is_empty() {
        println!("Currently no stored projects");
        Ok(())
    } else {
        for p in config.project_records.clone() {
            println!("{}", p.code);
        }
        
        Ok(())
    }
}

/// Iterates over all stored projects. The user can thus freely choose, for what code he want to add an activity
pub fn choose_project(projects: &[Project], prompt: &str) -> Result<Option<String>> {

    if projects.is_empty() {
        println!("Currently no stored projects.");
        return Err(anyhow!("No stored projects"));
    }

    let mut vec_of_strings: Vec<String> = projects.iter().map(|p| p.code.clone()).collect();

    vec_of_strings.push(String::from("INT"));
    vec_of_strings.push(String::from("Vacation"));


    let proj_entry = match Select::new(prompt, vec_of_strings).prompt(){
        Ok(project_entry)=> project_entry,
        Err(InquireError::OperationCanceled) | Err(InquireError::OperationInterrupted) => 
            return Ok(None),
        Err(e) => return Err(e.into()),
    };

    return Ok(Some(proj_entry));
}

/// Reports at the benning of the menu.
/// Tells the user the generall occupation of the month
pub fn base_report(config: &Config) -> Result<()> {

    let date = NaiveDate::from(Local::now().date_naive());

    let totals = filter_time_record_totals(config, date)?;

    // Print the report
    println!("Hour for {}/{}", date.month(), date.year());
    for (project, hours) in totals {
        println!("++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++");
        println!("Project: {}", project);
        println!("Assigned hours this month: {}", hours);
        println!("++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++");
        println!("\n");
    };

    return Ok(());
}

/// Function to iterate over a given month a return every entry for a project and the hours
/// assigned for this project
fn filter_time_record_totals(config: &Config, year: String, month: String) -> Result<HashMap<String,f64>> {

    let mut totals = HashMap::new();

    let date = NaiveDate::from_ymd_opt(year as i32, month, day)

    for record in config.time_records.iter().filter(|r| {
        r.date.month() == date.month() && r.date.year() == date.year()
    }) {
        for p_entry in &record.project_entries {
            *totals.entry(p_entry.project_name.code.clone()).or_insert(0.0) += p_entry.hours
        }
    };

    return Ok(totals);
}

/// Function to receive user input for a year he wants to have infomration for
fn choose_year(config: &Config, prompt: &str) -> Result<Option<String>> {
    
    let mut years_in_storage: Vec<String> = config.time_records.iter().map(|r| r.date.year().to_string()).collect();

    years_in_storage.sort_by(|a,b| b.cmp(a));
    years_in_storage.dedup();
    

    let entry = Select::new(prompt, years_in_storage).prompt_skippable()?;

    match entry {
        Some(year) => return Ok(Some(year)),
        None => return Ok(None)
    };
}

/// Function to receive user input for a month he want the information for
/// year should only 
fn choose_month(config: &Config, prompt: &str, year: i32) -> Result<Option<u32>> {

    // Consider using a struct instead, holding month name and numer.

    // Iterate over all of TimeRecords and get all the months in storage
    let mut month_in_storage_numerical: Vec<u32> = config.time_records
        .iter()
        .filter(|r| {
            r.date.year() == year
        })
        .map(|r| r.date.month())
        .collect();

    // Sort and deduplicate
    month_in_storage_numerical.sort_by(|a,b| b.cmp(a));
    month_in_storage_numerical.dedup();

    // Transform into actual month names
    let month_in_storage_string: Vec<String> = month_in_storage_numerical
        .iter()
        .map(|m| {
            let name = NaiveDate::from_ymd_opt(year, *m, 1)
            .unwrap()
            .format("%B")
            .to_string();
            name
        })
        .collect();
    
    // Get user selection
    let selection = Select::new(prompt, month_in_storage_string.clone())
        .prompt_skippable()?;

    let selected_month = match selection {
        Some(selection) => selection,
        None => return Ok(None),
    };

    // Turn back into usize
    let index = month_in_storage_string
        .iter()
        .position(|m| m == &selected_month)
        .with_context(|| format!("An error occurd! Month {}, could not be found", selected_month))?;

    Ok(Some(month_in_storage_numerical[index]))
}

