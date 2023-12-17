use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ResterauntLink {
    pub target: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ResterauntFilter {
    pub id: String,
    pub values: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ResterauntFiltering {
    pub filters: Vec<ResterauntFilter>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ResterauntItem {
    pub link: ResterauntLink,
    pub title: String,
    pub filtering: ResterauntFiltering,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ResterauntSection {
    pub items: Vec<ResterauntItem>,
    pub name: String,
    pub template: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GetAllRestaurantsResponse {
    pub sections: Vec<ResterauntSection>,
}
