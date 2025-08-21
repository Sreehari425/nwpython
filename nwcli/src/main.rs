use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: nwcli <source.nwpy> [--run]");
        process::exit(1);
    }
    let filename = &args[1];
    let run_flag = args.iter().any(|a| a == "--run");
    let source = match fs::read_to_string(filename) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading {}: {}", filename, e);
            process::exit(1);
        }
    };
    let tokens = nwparser::tokenize(&source);
    let py = nwtranspiler::transpile(&tokens);
    println!("{}", py);
    // Write to a .py file with the same base name as the input file
    let py_path = if let Some(pos) = filename.rfind('.') {
        format!("{}{}.py", &filename[..pos], "")
    } else {
        format!("{}.py", filename)
    };
    if let Err(e) = fs::write(&py_path, &py) {
        eprintln!("Error writing Python file: {}", e);
        process::exit(1);
    }
    if run_flag {
        let output = process::Command::new("python3")
            .arg(&py_path)
            .output();
        match output {
            Ok(out) => {
                println!("\n--- Python Output ---\n{}", String::from_utf8_lossy(&out.stdout));
                if !out.stderr.is_empty() {
                    println!("--- Python Error ---\n{}", String::from_utf8_lossy(&out.stderr));
                }
                if out.status.code().unwrap_or(0) != 0 {
                    println!("\n[Note] Interactive input (input()) is not supported in this compiler run. Please use hardcoded values for input or run the generated Python file manually in a terminal.");
                }
                println!("--- End ---");
            }
            Err(e) => {
                eprintln!("Error running Python: {}", e);
                process::exit(1);
            }
        }
    }
}
