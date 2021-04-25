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

// https://stackoverflow.com/a/38406885
fn title_case(s: String) -> String {
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

impl TryInto<Vec<String>> for InseeSource {
    type Error = anyhow::Error;

    fn try_into(self) -> anyhow::Result<Vec<String>> {
        log::info!("Reading {:?}...", &self.zip_filepath);
        let file = File::open(&self.zip_filepath)?;
        let mut zip_reader = zip::ZipArchive::new(file)?;
        assert!(zip_reader.len() == 1);
        let csv_file_reader = zip_reader.by_index(0)?;
        let mut csv = csv::ReaderBuilder::new()
            .delimiter(b';')
            .from_reader(csv_file_reader);

        log::info!("Parsing CSV data...");
        let rows: Vec<String> = csv
            .records()
            .map(Result::unwrap)
            .filter(|r| {
                // filter sex
                r.get(0).unwrap()
                    == match &self.sex {
                        Sex::MALE => "1",
                        Sex::FEMALE => "2",
                    }
            })
            .map(|r| r.get(1).unwrap().to_owned())
            .dedup()
            .filter(|n| n.len() > 1) // filter out single letters
            .filter(|n| *n != "_PRENOMS_RARES") // filter out source crap
            .map(|n| unidecode::unidecode(&n)) // normalize accents
            .dedup()
            .map(title_case)
            .collect();
        // TODO keep some other signals like frequency
        log::info!("Got {} names", rows.len());

        Ok(rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_title_case() {
        assert_eq!(title_case("BOB".to_string()), "Bob");
        assert_eq!(title_case("bob".to_string()), "Bob");
        assert_eq!(title_case("BOB-JOHN".to_string()), "Bob John");
    }
}
