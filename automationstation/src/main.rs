use std::collections::HashMap;
use std::{env};
use std::{thread, time};
use std::time::{SystemTime, UNIX_EPOCH};
use snailquote::unescape;

use regex::Regex;
use postgres::{Client, NoTls};
use powershell_script::PsScriptBuilder;
use urlencoding::encode;

fn main() {

    // *  Phase 0 - Monitor * //

    // Record executables runnning and determine weight by time and frequency of use. 
    // Runs for one day.
    let daily_processes = monitor_process();
    record_process(daily_processes);

    // Listen for mouse clicks, record mouse position and pixels in local area when clicked to determine how many times a day


    // * Phase 1 - Determine * //

    // 
}

/*************/
/* PROCESSES */
/*************/
fn monitor_process() -> HashMap<&'static str, f32> {

    // Get the current unix time
    let mut iteration = 1;
    let start = SystemTime::now();
    let since_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let seconds_since_epoch = since_epoch.as_secs();
    println!("Unixtimestamp: {:?}", seconds_since_epoch);

    // We want to compile regex as infrequently as possible
    let reg_title = Regex::new(r#"Name"#).unwrap();
    let reg_titlebar = Regex::new(r#"----"#).unwrap();
    let reg_process = Regex::new(r#"(?m)^.*?$"#).unwrap();

    let mut hashmap_processes: HashMap<&str, f32> = HashMap::new();

    for _x in 1..1441 {

        let vec_returnedlist = process_scan(&reg_title, &reg_titlebar, &reg_process);

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

        // Delay before next run
        println!("Iteration #{} completed...", iteration.to_string());
        let dur_minute = time::Duration::from_secs(60);
        let now = time::Instant::now();

        thread::sleep(dur_minute);
        assert!(now.elapsed() >= dur_minute);
        iteration += 1;
    }

    return hashmap_processes;

}
fn process_scan(reg_title: &Regex, reg_titlebar: &Regex, reg_process: &Regex) -> Vec<&'static str> {

    let mut vec_scannedlist: Vec<&str> = Vec::new();

    // Run a active window powershell script
    let ps = PsScriptBuilder::new()
        .no_profile(true)
        .non_interactive(true)
        .hidden(false)
        .print_commands(false)
        .build();
    let output = ps.run(r#"Get-Process | Where-Object {$_.MainWindowTitle -ne ""} | Select-Object Name"#).unwrap().to_string();

    // Remove excess strings
    let output = reg_title.replace(&output, "");
    let output = reg_titlebar.replace(&output, "");

    println!("Full output: {}", output);
    
    for process in reg_process.find_iter(&output) {
        println!("Adding new process: {:?}", process);
        vec_scannedlist.push(process.as_str().clone());
    }

    println!("Full list: {:?}", vec_scannedlist);
    return vec_scannedlist;
}
fn record_process(daily: HashMap<&'static str, f32>) {

    let mut daily_process = daily;

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
    let conn_string = format!("postgresql://{}:{}@192.168.0.103/postgres", pg_user, encodedpass);

    let mut client = match Client::connect(
        &conn_string,
        NoTls,
    ) {
        Ok(client) => client,
        Err(err) => {
            eprintln!("Error connecting to the database: {}", err);
            return ();
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

                if daily_process.contains_key(name) {
                    let new_weight_float = daily_process.get_key_value(name).unwrap().1;
                    let new_weight: f32 = new_weight_float.to_owned();
                    let _update = client.execute(r#"UPDATE process SET weight = "$1" AND lastactive = "$2" WHERE name = "$3""#, &[&new_weight, &seconds_since_epoch, &name],);

                    daily_process.remove(name);
                }
            };
        }
        Err(err) => {
            eprintln!("Error during process select query: {}", err);
            return ();
        }
    }

    // Insert new processes
    for process in &daily_process {
        println!("Process name: {}", process.0);
        println!("Process key: {}", process.1);
        let _insert = client.execute("INSERT INTO process (name, weight, lastactive, firstactive) VALUES ($1, $2, $3, $4)", &[process.0, process.1, &seconds_since_epoch, &seconds_since_epoch],);
    }
}