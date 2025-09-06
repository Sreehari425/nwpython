use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    // Help flag handling
    let help_flag = args.iter().any(|a| a == "-h" || a == "--help");
    if help_flag {
        print_help();
        return;
    }
    if args.len() < 2 {
        print_help();
        process::exit(1);
    }
    let filename = &args[1];
    let run_flag = args.iter().any(|a| a == "--run");
    let reverse_flag = args.iter().any(|a| a == "--reverse-transpile");
    let format_flag = args.iter().any(|a| a == "--format");
    let in_place_flag = args.iter().any(|a| a == "--in-place");
    let source = match fs::read_to_string(filename) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading {}: {}", filename, e);
            process::exit(1);
        }
    };
    let is_py_input = filename.ends_with(".py");

    // Formatter-only mode: format NWPython source and print or write
    if format_flag && !is_py_input && !reverse_flag {
        let formatted = nwformatter::format_nwpython(&source);
        if in_place_flag {
            if let Err(e) = fs::write(filename, &formatted) {
                eprintln!("Error writing formatted file: {}", e);
                process::exit(1);
            }
        }
        println!("{}", formatted);
        return;
    }

    if reverse_flag || is_py_input {
        // Reverse transpile Python -> NWPython
        let mut nw_code = nwtranspiler::reverse_transpiler::reverse_transpile(&source);
        if format_flag {
            nw_code = nwformatter::format_nwpython(&nw_code);
        }
        println!("{}", nw_code);
        // Save as .nwpy next to input
        let nw_path = if let Some(pos) = filename.rfind('.') {
            format!("{}{}.nwpy", &filename[..pos], "")
        } else {
            format!("{}.nwpy", filename)
        };
        if let Err(e) = fs::write(&nw_path, &nw_code) {
            eprintln!("Error writing NWPython file: {}", e);
            process::exit(1);
        }
        // Don't run reverse-transpiled code
        return;
    }

    // Regular transpile: NWPython -> Python
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
        let output = process::Command::new("python3").arg(&py_path).output();
        match output {
            Ok(out) => {
                println!(
                    "\n--- Python Output ---\n{}",
                    String::from_utf8_lossy(&out.stdout)
                );
                if !out.stderr.is_empty() {
                    println!(
                        "--- Python Error ---\n{}",
                        String::from_utf8_lossy(&out.stderr)
                    );
                }
                if out.status.code().unwrap_or(0) != 0 {
                    println!(
                        "\n[Note] Interactive input (input()) is not supported in this compiler run. Please use hardcoded values for input or run the generated Python file manually in a terminal."
                    );
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

fn print_help() {
    println!("nwcli — NWPython toolchain CLI\n");
    println!("Usage: nwcli <source.nwpy|source.py> [options]");
    println!("Options:");
    println!("  -h, --help                Show this help message and exit");
    println!("      --run                 After transpiling NWPython -> Python, run the generated Python");
    println!("      --reverse-transpile   Convert Python -> NWPython (saves .nwpy next to input)");
    println!("      --format              Run the NWPython formatter on reverse-transpile output or on .nwpy input");
    println!("      --in-place            When used with --format, overwrite the input file with formatted output");
    println!("\nExamples:");
    println!("  nwcli source.nwpy           # transpile to Python and write source.py");
    println!("  nwcli source.nwpy --run     # transpile and run the generated Python");
    println!("  nwcli main.py --reverse-transpile --format  # produce formatted main.nwpy");
}
