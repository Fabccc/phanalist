use analyse::Analyse;
use outputs::Format;
mod analyse;
mod config;
mod file;
mod outputs;
mod results;
mod rules;
mod indexers;

pub fn scan(path: String) -> results::Results {
    let output_format = Format::json;
    let config = Analyse::parse_config(path.clone(), &output_format, false);

    let analyze: Analyse = Analyse::new(&config);

    analyze.scan("./".to_string(), &config, false, &output_format)
}

#[test]
fn run() {
    let violations = scan(String::from("./rules/examples/phanalist.yaml"));

    assert_ne!(0, violations.total_files_count)
}
