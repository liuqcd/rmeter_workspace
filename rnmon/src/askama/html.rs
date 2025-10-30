use askama::Template;

#[derive(Template)]
#[template(path = "nmon/index_askama.html.j2", escape = "none")] 
pub struct NmonHtmlTemplate { 
    nmonfiles: Vec<NmonFile>, 
    charts: Vec<Charts>,
    // nmonjs: Vec<String>,
}
impl NmonHtmlTemplate {
    pub fn new(nmonfiles: Vec<NmonFile>, charts: Vec<Charts>) -> Self {
        Self {
            nmonfiles,
            charts,
        }
    }
}

pub struct NmonFile {
    id: String,
    name: String,
    selected: String,
}
impl NmonFile {
    pub fn new(id: String, name: String, selected: String) -> Self {
        Self {
            id,
            name,
            selected,
        }
    }
}

pub struct Charts {
    name: String,
    selected: String,
}
impl Charts {
    pub fn new(name: String, selected: String) -> Self {
        Self {
            name,
            selected,
        }
    }
}
