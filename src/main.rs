use std::convert::TryInto;

mod source;

fn main() {
    // Init logger
    simple_logger::SimpleLogger::new().init().unwrap();

    // Get names
    let source = source::InseeSource::new().expect("Failed to initialize data source");
    let names: Vec<String> = source.try_into().expect("Failed to build names");

    println!("{:?}", names);
}
