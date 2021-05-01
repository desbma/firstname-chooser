use std::convert::TryInto;
use std::fs::File;
use std::io::copy;
use std::path::PathBuf;

use heck::TitleCase;
use itertools::Itertools;

use crate::Sex;

pub struct InseeSource {
    zip_filepath: PathBuf,
    sex: Sex,
}

const MIN_YEAR: u16 = 1970;

// https://stackoverflow.com/a/38406885
fn title_case(s: &str) -> String {
    // we don't use Inflector because of https://github.com/whatisinternet/Inflector/issues/79

    // let mut c = s.chars();
    // match c.next() {
    //     None => String::new(),
    //     Some(f) => f.to_uppercase().collect::<String>() + &c.as_str().to_lowercase(),
    // }

    s.to_title_case()
}

// https://www.insee.fr/fr/statistiques/2540004?sommaire=4767262
const INSEE_ZIP_URL: &str = "https://www.insee.fr/fr/statistiques/fichier/2540004/nat2019_csv.zip";
const INSEE_ZIP_FILENAME: &str = "nat2019_csv.zip";

impl InseeSource {
    pub fn new(sex: &Sex) -> anyhow::Result<InseeSource> {
        let binary_name = env!("CARGO_PKG_NAME");
        let xdg_dirs = xdg::BaseDirectories::with_prefix(binary_name)?;
        let zip_filepath = match xdg_dirs.find_cache_file(INSEE_ZIP_FILENAME) {
            Some(fp) => fp,
            None => {
                log::info!("{}", INSEE_ZIP_URL);
                let mut response = reqwest::blocking::get(INSEE_ZIP_URL)?.error_for_status()?;
                let zip_filepath = xdg_dirs.place_cache_file(INSEE_ZIP_FILENAME)?;
                let mut zip_file = File::create(&zip_filepath)?;
                copy(&mut response, &mut zip_file)?;
                zip_filepath
            }
        };

        Ok(InseeSource {
            zip_filepath,
            sex: sex.to_owned(),
        })
    }
}

impl TryInto<(Vec<String>, Vec<f64>)> for InseeSource {
    type Error = anyhow::Error;

    fn try_into(self) -> anyhow::Result<(Vec<String>, Vec<f64>)> {
        log::info!("Reading {:?}...", &self.zip_filepath);
        let file = File::open(&self.zip_filepath)?;
        let mut zip_reader = zip::ZipArchive::new(file)?;
        assert!(zip_reader.len() == 1);
        let csv_file_reader = zip_reader.by_index(0)?;
        let mut csv = csv::ReaderBuilder::new()
            .delimiter(b';')
            .from_reader(csv_file_reader);

        log::info!("Parsing CSV data...");
        let processed_rows: Vec<_> = csv
            .records()
            .map(Result::unwrap)
            .filter(|r| {
                // Filter sex
                r.get(0).unwrap()
                    == match &self.sex {
                        Sex::MALE => "1",
                        Sex::FEMALE => "2",
                    }
            })
            .filter(|r| {
                let name = r.get(1).unwrap();
                name.len() > 1  // Filter out single letters
                && name != "_PRENOMS_RARES" // Filter out source crap
            })
            .filter(|r| {
                r.get(2).unwrap().parse::<u16>().unwrap_or(0) >= MIN_YEAR // Filter out old years
            })
            .map(|r| {
                (
                    unidecode::unidecode(&title_case(&r.get(1).unwrap())), // Normalize case & accents
                    r.get(3).unwrap().parse::<usize>().unwrap(),           // Parse freq
                )
            })
            .collect();

        let names: Vec<String> = processed_rows
            .iter()
            .map(|r| r.0.to_owned())
            .dedup()
            .collect();
        let freq_max = processed_rows
            .iter()
            .group_by(|r| r.0.to_owned())
            .into_iter()
            .map(|(_, fs)| fs.map(|e| e.1).sum::<usize>())
            .max()
            .unwrap();
        let weightings: Vec<f64> = processed_rows
            .iter()
            .group_by(|r| r.0.to_owned())
            .into_iter()
            .map(|(_, fs)| fs.map(|e| e.1).sum::<usize>() as f64 / freq_max as f64)
            .collect();

        debug_assert!(weightings.iter().all(|w| (&0.0..=&1.0).contains(&w)));

        assert_eq!(names.len(), weightings.len());

        log::info!("Got {} names", names.len());

        Ok((names, weightings))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_title_case() {
        assert_eq!(title_case("BOB"), "Bob");
        assert_eq!(title_case("bob"), "Bob");
        assert_eq!(title_case("BOB-JOHN"), "Bob John");
    }
}
