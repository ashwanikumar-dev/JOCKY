use compiler::compile;
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!(
            "Usage: {} <script-file> <investigation-id>",
            args.first().map(String::as_str).unwrap_or("compiler")
        );
        process::exit(2);
    }

    let script_path = &args[1];
    let investigation_id = &args[2];

    let source = match fs::read_to_string(script_path) {
        Ok(source) => source,
        Err(err) => {
            eprintln!(
                "Failed to read JOCKY script '{}': {}",
                script_path, err
            );
            process::exit(1);
        }
    };

    match compile(&source, investigation_id) {
        Ok(ir) => {
            match serde_json::to_string_pretty(&ir) {
                Ok(json) => {
                    println!("{json}");
                }
                Err(err) => {
                    eprintln!("Failed to serialize IR: {err}");
                    process::exit(1);
                }
            }
        }

        Err(err) => {
            eprintln!("Compilation failed: {err}");
            process::exit(1);
        }
    }
}