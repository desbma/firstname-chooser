#![feature(test)]
extern crate test;

use arg_enum_proc_macro::ArgEnum;
use std::convert::TryInto;
use structopt::StructOpt;

mod graph;
mod source;

#[derive(ArgEnum, Clone, Debug)]
pub enum Sex {
    MALE,
    FEMALE,
}

#[derive(StructOpt, Debug)]
#[structopt(version=env!("CARGO_PKG_VERSION"), about="Help you finding a name for a future human being.")]
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

    // Command line args
    let opts = CommandLineOpts::from_args();
    log::trace!("{:?}", opts);

    // Get names
    let source = source::InseeSource::new(&opts.sex).expect("Failed to initialize data source");
    let names: Vec<String> = source.try_into().expect("Failed to build names");
    log::debug!("{:?}", names);

    // Build graph
    let mut graph = graph::NameGraph::new();
    graph.fill(&names);
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn bench_fill_graph(b: &mut Bencher) {
        let source = source::InseeSource::new(&Sex::MALE).unwrap();
        let names: Vec<String> = source.try_into().unwrap();

        b.iter(|| {
            let mut graph = graph::NameGraph::new();
            graph.fill(&names);
        });
    }
}
