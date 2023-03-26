use std::path::PathBuf;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PersonData {
    pub imdb_id: String,
    pub name: String,
    pub wiki_link: Option<String>,
    pub father_name: Option<String>,
    pub father_imdb_id: Option<String>,
    pub father_wiki_link: Option<String>,
    pub mother_name: Option<String>,
    pub mother_imdb_id: Option<String>,
    pub mother_wiki_link: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub enum NepoBabiness {
    No,
    OnlyMother,
    OnlyFather,
    Yes,
}

pub fn get_nepos_csv_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.push("data");
    path.push("nepos.csv");
    return path;
}

pub fn determine_nepo_babiness(person: &PersonData) -> NepoBabiness {
    if person.mother_wiki_link.is_some() && person.father_wiki_link.is_some() {
        NepoBabiness::Yes
    } else if person.mother_wiki_link.is_some() {
        NepoBabiness::OnlyMother
    } else if person.father_wiki_link.is_some() {
        NepoBabiness::OnlyFather
    } else {
        NepoBabiness::No
    }
}
