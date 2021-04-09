use arg_enum_proc_macro::ArgEnum;
use std::convert::TryInto;
use structopt::StructOpt;

mod source;

#[derive(ArgEnum, Debug)]
pub enum Sex {
    MALE,
    FEMALE,
}

#[derive(StructOpt, Debug)]
#[structopt(version=env!("CARGO_PKG_VERSION"), about="Help you naming a future being.")]
pub struct CommandLineOpts {
    /// Sex
    #[structopt(
        short,
        long,
        possible_values = &Sex::variants(),
        case_insensitive = true
    )]
    pub sex: Sex,
}

fn main() {
    // Init logger
    simple_logger::SimpleLogger::new().init().unwrap();

    // Command linea args
    let opts = CommandLineOpts::from_args();
    log::trace!("{:?}", opts);

    // Get names
    let source = source::InseeSource::new().expect("Failed to initialize data source");
    let names: Vec<String> = source.try_into().expect("Failed to build names");
    log::trace!("{:?}", names);
}
