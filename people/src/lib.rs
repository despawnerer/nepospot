mod people;

pub use people::PEOPLE;

#[derive(Debug, serde::Serialize)]
pub struct StaticPersonData {
    pub imdb_id: &'static str,
    pub name: &'static str,
    pub wiki_link: Option<&'static str>,
    pub father_name: Option<&'static str>,
    pub father_imdb_id: Option<&'static str>,
    pub father_wiki_link: Option<&'static str>,
    pub mother_name: Option<&'static str>,
    pub mother_imdb_id: Option<&'static str>,
    pub mother_wiki_link: Option<&'static str>,
}
