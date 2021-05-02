#![feature(test)]
extern crate test;

use std::collections::HashMap;
use std::convert::TryInto;

use arg_enum_proc_macro::ArgEnum;
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

    /// Min name length
    #[structopt(short, long, default_value = "3")]
    pub min_name_lenth: u8,

    /// Exclude compound names
    #[structopt(short, long)]
    pub exclude_compound: bool,

    /// Exclude data below this year for frequency weighting
    #[structopt(long)]
    pub min_year: Option<u16>,

    /// How much to favour common names compared to rare ones
    #[structopt(short, long, default_value = "0.5")]
    pub commonness_factor: f64,
}

fn main() {
    // Init logger
    simple_logger::SimpleLogger::new().init().unwrap();

    // Command line args
    let opts = CommandLineOpts::from_args();
    log::trace!("{:?}", opts);

    // Get names
    let source = source::InseeSource::new(
        &opts.sex,
        opts.min_name_lenth,
        opts.exclude_compound,
        opts.min_year,
    )
    .expect("Failed to initialize data source");
    let (names, weightings): (Vec<String>, Vec<f64>) =
        source.try_into().expect("Failed to build names");
    log::debug!("{:#?}", names);
    log::debug!("{:#?}", weightings);

    // Build graph
    let mut graph = graph::LevenshteinGraph::new();
    graph.fill(&names);

    // Main loop
    let mut prev_choices: HashMap<usize, bool> = HashMap::new();
    let str_choices = vec![
        "Hell yeah!",
        "Errh.. nope",
        "No, but suggest a similar name",
        "Remind me of my previous choices",
    ];
    let mut cur_idx = graph.random();
    loop {
        // User input
        let choice = dialoguer::Select::new()
            .with_prompt(format!(
                "What do you think of the name {:?}?",
                names[cur_idx]
            ))
            .items(&str_choices)
            .default(1)
            .interact()
            .unwrap();

        // React to choice
        match choice {
            0 | 1 => {
                prev_choices.insert(cur_idx, choice == 0);
            }
            2 => {
                prev_choices.insert(cur_idx, false);
                cur_idx = graph.closest(cur_idx, &prev_choices);
                continue;
            }
            3 => {
                for (i, (idx, liked)) in prev_choices.iter().enumerate() {
                    eprintln!(
                        "#{:02} {} {:?}",
                        i,
                        if *liked { '+' } else { '-' },
                        names[*idx]
                    );
                }
                continue;
            }
            _ => unreachable!(),
        }

        // Next recommandation
        cur_idx = graph.recommend(&prev_choices, &weightings, opts.commonness_factor);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn bench_fill_graph(b: &mut Bencher) {
        let source = source::InseeSource::new(&Sex::MALE).unwrap();
        let (names, _): (Vec<String>, Vec<f64>) = source.try_into().unwrap();

        b.iter(|| {
            let mut graph = graph::LevenshteinGraph::new();
            graph.fill(&names);
        });
    }
}
