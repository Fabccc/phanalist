extern crate exitcode;

use std::path::Path;
use std::process;
use std::str::FromStr;

use clap::{arg, Parser};
use mago_interner::ThreadedInterner;
use mago_php_version::PHPVersion;
use mago_source::SourceManager;
use php_parser_rs::printer::print;

use crate::analyse::Analyse;
use crate::outputs::Format;

mod analyse;
mod ast;
mod config;
mod file;
mod outputs;
mod results;
mod rules;

///
/// A static analyser for your PHP project.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
struct Args {
    #[arg(short, long, default_value = "./phanalist.yaml")]
    config: String,
    #[arg(short, long, default_values_t = ["./src".to_string()])]
    src: Vec<String>,
    #[arg(short, long)]
    /// The list of rules to use (by default it is used from config)
    rules: Option<Vec<String>>,
    #[arg(short, long, default_value = "text")]
    /// Possible options: text, json, gitlab
    output_format: String,
    #[arg(long)]
    /// Output only summary
    summary_only: bool,
    #[arg(short, long)]
    /// Do not output the results
    quiet: bool,
}

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");
    let args = Args::parse();

    let quiet = args.quiet;

    let paths = args.src;
    for path in paths.iter() {
        if !Path::new(&path).exists() {
            println!("Path {} does not exist", path);
            process::exit(exitcode::IOERR);
        }
    }

    let format = match outputs::Format::from_str(args.output_format.as_str()) {
        Ok(format) => format,
        Err(_) => {
            println!("Invalid input format ({})", args.output_format.as_str());
            process::exit(exitcode::USAGE);
        }
    };
    let threaded_interner = ThreadedInterner::new();

    let source_manager = match ast::load(&paths.first().unwrap(), &threaded_interner) {
        Ok(source) => source,
        Err(e) => {
            println!("Error loading source: {}", e);
            process::exit(exitcode::IOERR);
        }
    };

    println!(
        "Loaded {} source files from {}",
        source_manager.len(),
        paths.first().unwrap()
    );

    // TODO: Don't hardcore PHP version, either read from config or fallback to the environment variable (or opening a process)
    let ast = match ast::build_ast(&threaded_interner, &source_manager, PHPVersion::PHP84) {
        Ok(ast) => ast,
        Err(e) => {
            println!("Error building AST: {}", e);
            process::exit(exitcode::SOFTWARE);
        }
    };

    println!("Built AST with {} files", ast.tree.len());

    let mut config = Analyse::parse_config(args.config, &format, quiet);
    if let Some(rules) = args.rules {
        config.enabled_rules = rules;
    }
    let mut analyze = Analyse::new(&config);

    let mut has_violations = false;

    for path in paths.iter() {
        let mut results = analyze.scan(
            path.clone(),
            &config,
            format != Format::json && !quiet,
            &format,
        );
        if !quiet {
            analyze.output(&mut results, format.clone(), args.summary_only);
        }
        has_violations = has_violations || results.has_any_violations();
    }

    if has_violations {
        process::exit(exitcode::SOFTWARE);
    } else {
        process::exit(exitcode::OK);
    }
}
