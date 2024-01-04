use std::collections::HashMap;
use std::env;
use std::process::Command;
use std::{thread, time};
use std::time::{SystemTime, UNIX_EPOCH};

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
    let mut iteration = 1;
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

    for _x in 1..1441 {

        let vec_returnedlist = process_scan(&reg_process, &reg_format);

        for returnedlist in vec_returnedlist {
            // Weight calculation - 8 hours of uptime is '1', everything else is a median
            let weight: f32 = 0.0020833;
            if hashmap_processes.contains_key(&returnedlist) {
                *hashmap_processes.get_mut(&returnedlist).unwrap() += weight;
                println!("Update to existing: {:?} with a new weight of: {:?}", &returnedlist, hashmap_processes.get(&returnedlist).unwrap());
            }
            else {
                println!("Adding : {:?}", &returnedlist);
                hashmap_processes.insert(
                    returnedlist,
                    weight,
                );
            }
        }
        // Get the current unix time
        let start = SystemTime::now();
        let since_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let seconds_since_epoch: i64 = since_epoch.as_secs().try_into().unwrap();
        println!("Unixtimestamp: {:?}", seconds_since_epoch);

        // Get the env variables
        let pg_user = match env::var_os("PG_USER") {
            Some(v) => v.into_string().unwrap(),
            None => panic!("$USER is not set")
        };
        let pg_pass = match env::var_os("PG_PASS") {
            Some(v) => v.into_string().unwrap(),
            None => panic!("$PASS is not set")
        };

        // DB Connection
        let encodedpass = encode(&pg_pass);
        let conn_string = format!("postgresql://{}:{}@localhost/postgres", pg_user, encodedpass);

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

        // Update existing processes
        let result = client.query("SELECT * FROM process", &[]);
        match result {
            Ok(rows) => {
                for row in rows {
                    let name: &str = row.get(0);
                    let weight: f32 = row.get(1);
                    let lastactive: i64 = row.get(2);
                    let firstactive: i64 = row.get(3);
                    
                    println!("Found row: {:?}, {:?}, {:?}, {:?}", name, weight, lastactive, firstactive);

                    if hashmap_processes.contains_key(name) {
                        let new_weight_float = hashmap_processes.get_key_value(name).unwrap().1;
                        let new_weight: f32 = new_weight_float.to_owned();
                        let mut update = client.execute(r#"UPDATE process SET weight = "$1" AND lastactive = "$2" WHERE name = "$3""#, &[&new_weight, &seconds_since_epoch, &name],)?;

                        hashmap_processes.remove(name);
                    }
                };
            }
            Err(err) => {
                eprintln!("Error during process select query: {}", err);
                return Err(err);
            }
        }

        // Insert new processes
        for process in &hashmap_processes {
            println!("Process name: {}", process.0);
            println!("Process key: {}", process.1);
            let mut insert = client.execute("INSERT INTO process (name, weight, lastactive, firstactive) VALUES ($1, $2, $3, $4)", &[process.0, process.1, &seconds_since_epoch, &seconds_since_epoch],)?;
        }

        // Delay before next run
        println!("Iteration #{} completed...", iteration.to_string());
        let dur_minute = time::Duration::from_secs(60);
        let now = time::Instant::now();

        thread::sleep(dur_minute);
        assert!(now.elapsed() >= dur_minute);
        iteration += 1;
    }
    Ok(())

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