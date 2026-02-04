use chrono::{NaiveDate, NaiveTime};
use inquire::Confirm;
use serde::de::DeserializeOwned;
use serde::{Serialize, Deserialize};
// use crate::storage::*;
use anyhow::{Context, Result};
use std::path::{PathBuf};
use std::{fs,env};


/// A Time Record of a day. Summarizes start, end and pause of a worker as well as
/// their activities for multiple projects
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimeRecord {
    pub date: NaiveDate,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub pause_minutes: f64,
    pub project_entries: Vec<ProjectEntry> // Can be initialized as an empty vec![]. Will be pushed with ProjectEntries!
}

impl TimeRecord{
    /// rounds to the neares quarter of a value
    pub fn round_quarter(h: f64) -> f64 {
        (h * 4.0).round() / 4.0
    }
    /// Already substracts the pause from a given workday
    pub fn get_net_hours(&self) -> f64 {

        let duration = self.end_time.signed_duration_since(self.start_time);
        let seconds = duration.num_seconds();
        let hours = seconds as f64 / 3600.0;

        let net = Self::round_quarter(hours.max(0.0)) - self.pause_minutes;
        net
    }
    /// Gets the already allocated hours of a workday
    pub fn allocated_hours(&self) -> f64 {
        self.project_entries.iter().map(|e| e.hours).sum()
    }
    /// Gets the hours free for assignment for a given workday
    pub fn remaining_hours(&self) -> f64 {
        self.get_net_hours() - self.allocated_hours()
    }
    /// Prints the already allocated projects and time windows for a project
    pub fn print_already_recorded(&self) -> (){
        for entry in &self.project_entries {
            println!("- {:?}, Allocated: {}", entry.project_name.code, entry.hours);
        }
    }
    /// Checks, if there is already an entry for a TimeRecord
    pub fn prohibit_duplicate_entry(&self, project_code: &str) -> bool {


        self.project_entries.iter().any(|e| &e.project_name.code == project_code)
   
    }
}


/// Struct to store the project, the time frame and the type of activity
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectEntry {
    pub project_name: Project,
    pub hours: f64,
    pub activity: String
}

/// Struct to store Projects of a user in memory (json)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub code: String, // Should be changed to &str
    pub allocation: f64
}

impl Project {
    pub fn check_empty(&self) -> bool {
        return self.code.is_empty();
    }
}

/// Storage of timeRecords and ProjectRecords for the CLI
/// Type:
///     time_records: Vec<TimeRecord>
///     project_records: Vec<Project>
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub time_records: Vec<TimeRecord>,
    pub project_records: Vec<Project>,
    pub t_path: PathBuf,
    pub p_path: PathBuf
    //ADD t_path and p_path as Str
}

/// Config to be used from ptt_cli
impl Config{

    /// Builds the initial Config struct at the start of the programm
    pub fn build(t_name: &str, p_name: &str) -> Result<Config> {
        // Intended to load all the data.

        //let time_records: Vec<TimeRecord> = load_data(t_name);
        //let project_records: Vec<Project> = load_data(p_name);
        let time_records: Vec<TimeRecord> = Config::load(t_name)?;
        let project_records: Vec<Project> = Config::load(p_name)?;

        let t_path = Config::get_local_config_path(t_name)?;
        let p_path = Config::get_local_config_path(p_name)?;

        Ok(Config{time_records, project_records, t_path, p_path})
    }

    /// Adds a time Record to Config.time_records
    pub fn add_time_record(&mut self, new_record: &TimeRecord)-> Result<()>{
        //Call Save after usage
        self.time_records.push(new_record.clone());
        self.save()?;
        println!("Time Record added");
        Ok(())
        
    }

    /// Adds a Project to Config.project_records
    pub fn add_project(&mut self, new_project: Project)-> Result<()>{
        //Call Save after usage
        self.project_records.push(new_project);
        self.save()?;
        println!("Project added");
        Ok(())

    }

    /// Adds a ProjectEntry to my TimeRecord stored in my Config.time_records
    pub fn add_project_entry(&mut self, date: NaiveDate, new_project_entry: ProjectEntry)-> Result<()>{
        if let Some(record) = self.time_records.iter_mut().find(|r| r.date == date) {
            /*
            if record.prohibit_duplicate_entry(&new_project_entry.project_name.code){
                println!("A record for this project already exists");
                return Err(anyhow!("Duplicate entry for record {:#?}", record));
            } else {
                record.project_entries.push(new_project_entry);
                self.save()?;
                println!("Project entry added!");
                return Ok(());
            }
            */

            record.project_entries.push(new_project_entry);
            self.save()?;
            println!("Project entry added!");
            return Ok(());
        }
        return Ok(());
    }

    /// Deletes a Project from Config.project_records
    pub fn delete_project(&mut self, to_delete: String) -> Result<()> {

        if self.project_records.is_empty(){
            println!("Currently no stored projects");
            return Ok(());
        }

        self.project_records.retain(|p| p.code != to_delete);
        self.save()?;
        println!("Project deleted");
        Ok(())
    }

    pub fn delete_time_record(&mut self, date: NaiveDate) -> Result<()> {

        let confirm = Confirm::new(&format!("Are you sure you want to delete the record for date: {}", date))
            .prompt()?;

        if matches!(confirm, true){
            self.time_records.retain(|r| r.date != date);
            self.save()?;
            println!("Time Record deleted");
            return Ok(());
            } else {
                println!("Record NOT deleted");
                return Ok(());
            };

    }

    pub fn list_stored(&self) -> Result<()>{
        // Try making it so I can use it on all types
        todo!()
    }

    pub fn load<T: DeserializeOwned>(file_name: &str)-> Result<Vec<T>>{

        let p = Config::get_local_config_path(file_name)?;

        if p.exists() {
            let contents = fs::read_to_string(&p)
                .with_context(|| format!("Failed to read path: {:#?}", p))?;

            if contents.trim().is_empty() {
                return Ok(Vec::new())
            }

            let data: Vec<T> = serde_json::from_str(&contents)
                .with_context(|| format!("Failed to deserialze data for path: {:#?}", p))?;
            Ok(data)
        } else {
            println!("First Run! Creating File: {}", file_name);
            fs::File::create(p)
                .with_context(|| format!("Failed to create file: {}", file_name))?;

            return Ok(Vec::new());
            
        }

    }

    pub fn save(&self)-> Result<()> {
        //let file_path = self.get_local_config_path();
        
        if let Ok(t_json) = serde_json::to_string_pretty(&self.time_records)
            .context("Failed to seialize time_records") {
                fs::write(&self.t_path, t_json)
                    .with_context(|| format!("Failed to write TimeRecords JSON to {:#?}", &self.t_path))?;
            }; 
        if let Ok(p_json) = serde_json::to_string_pretty(&self.project_records)
            .context("Failed to serialize projects") {
                fs::write(&self.p_path, p_json)
                    .with_context(|| format!("Failed to write Project Records JSON to {:#?}", &self.p_path))?;
            };
        
        println!("Data saved sucessfully");
        Ok(())
    }

    pub fn get_local_config_path(file_name: &str) -> Result<PathBuf> {
        let mut path = env::current_dir()
            .context("Failed to retrieve current dir!")?;
        path.push(file_name);
        Ok(path)
        //todo: File names will be inferred by config
    }
}

pub struct MonthChoice {
    month_name: String,
    month_number: u32,
}