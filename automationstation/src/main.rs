use std::collections::HashMap;
use std::process::Command;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use std::thread;

use regex::Regex;
use postgres::{Client, NoTls, Error};
use urlencoding::encode;

fn main() {

    // *  Phase 0 - Monitor * //

    // Record executables runnning and determine weight by time and frequency of use. 
    // Runs for one day.
    monitor_process();

    // Listen for mouse clicks, record mouse position and pixels in local area when clicked to determine how many times a day


    // * Phase 1 - Determine * //

    // 
}

fn monitor_process() -> Result<(), Error> {

    // Get the current unix time
    let start = SystemTime::now();
    let since_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let seconds_since_epoch = since_epoch.as_secs();
    println!("Unixtimestamp: {:?}", seconds_since_epoch);

    // We want to compile regex as infrequently as possible
    let reg_process = Regex::new(r#"".*exe"#).unwrap();
    let reg_format = Regex::new(r#"\""#).unwrap();

    let mut hashmap_processes: HashMap<String, f32> = HashMap::new();

    // for _x in 1..1441 {
    //     let vec_returnedlist = process_scan(&reg_process, &reg_format);
    //     for returnedlist in vec_returnedlist {

    //         // Weight calculation - 8 hours of uptime is '1', everything else is a median
    //         let weight: f32 = 0.0020833;
    //         if hashmap_processes.contains_key(&returnedlist) {
    //             *hashmap_processes.get_mut(&returnedlist).unwrap() += weight;
    //             println!("Update to existing: {:?} with a new weight of: {:?}", &returnedlist, hashmap_processes.get(&returnedlist).unwrap());
    //         }
    //         else {
    //             println!("Adding : {:?}", &returnedlist);
    //             hashmap_processes.insert(
    //                 returnedlist,
    //                 weight,
    //             );
    //         }
    //     }

    //     thread::sleep(Duration::from_secs(60));
    // }
    
    // Get the current unix time
    let start = SystemTime::now();
    let since_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let seconds_since_epoch = since_epoch.as_secs();
    println!("Unixtimestamp: {:?}", seconds_since_epoch);

    // DB Storage for processes
    let pgpass = 
    let encodedpass = 
    let conn_string = 

    let mut client = match Client::connect(
        &conn_string,
        NoTls,
    ) {
        Ok(client) => client,
        Err(err) => {
            eprintln!("Error connecting to the database: {}", err);
            return Err(err);
        }
    };
    let result = client.query("SELECT * FROM process", &[]);
    match result {
        Ok(rows) => {
            for row in rows {
                let name: &str = row.get(0);
                let weight: f32 = row.get(1);
                let lastactive: i64 = row.get(2);
                let firstactive: i64 = row.get(3);
                
                println!("Found row: {:?}, {:?}, {:?}, {:?}", name, weight, lastactive, firstactive);

                    // Updating a known process
                    if hashmap_processes.contains_key(name) {
                        let new_weight = hashmap_processes.get(name).unwrap();
                        let new_lastactive = seconds_since_epoch;
                        hashmap_processes.remove(name);
                    }
                    // Adding a new process to the DB
                    else {

                    }

            };
        }
        Err(err) => {
            eprintln!("Error during select query: {}", err);
            return Err(err);
        }
    }

    Ok(())

    // Store our processes and weights

}
fn process_scan(reg_process: &Regex, reg_format: &Regex) -> Vec<String> {

    let mut vec_scannedlist: Vec<String> = Vec::new();

    let command = "tasklist";
    let args = ["/nh", "/fo", "csv", "/fi", "STATUS eq running"];

    let output = Command::new(command)
        .args(&args)
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        eprintln!("Command failed with: {:?}", String::from_utf8_lossy(&output.stderr));
    } else {
        let output_raw = String::from_utf8_lossy(&output.stdout);
        for process in reg_process.find_iter(&output_raw) {
            let process_clean = reg_format.replace_all(&process.as_str(), "");

            // Ensure no duplicates on each instance
            if vec_scannedlist.contains(&process_clean.to_string()) {
            }
            else {
                vec_scannedlist.push(process_clean.to_string());
            }
        }
    }
    return vec_scannedlist;
}