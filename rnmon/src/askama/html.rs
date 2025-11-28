use askama::Template;

#[derive(Template)]
#[template(path = "index_askama.html.jinja2", escape = "none")]
pub struct NmonHtmlTemplate {
    nmonfiles: Vec<NmonFile>,
    charts: Vec<Chart>,
    // nmonjs: Vec<String>,
}
impl NmonHtmlTemplate {
    pub fn new(nmonfiles: Vec<NmonFile>, charts: Vec<Chart>) -> Self {
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

#[derive(Clone, Debug)]
pub enum ChartsName {
    SYS_SUMM,
    CPU_SUMM,
    CPU_ALL,
    JFSFILE,
    MEM_FREE,
    MEM_ACTIVE,
    MEM_SWAP,
    DISKBUSY_AWMN,
}

impl ToString for ChartsName {
    fn to_string(&self) -> String {
        match self {
            ChartsName::SYS_SUMM => "SYS_SUMM".to_string(),
            ChartsName::CPU_SUMM => "CPU_SUMM".to_string(),
            ChartsName::CPU_ALL => "CPU_ALL".to_string(),
            ChartsName::JFSFILE => "JFSFILE".to_string(),
            ChartsName::MEM_FREE => "MEM_FREE".to_string(),
            ChartsName::MEM_ACTIVE => "MEM_ACTIVE".to_string(),
            ChartsName::MEM_SWAP => "MEM_SWAP".to_string(),
            ChartsName::DISKBUSY_AWMN => "DISKBUSY_AWMN".to_string(),
        }
    }

}

pub struct Chart {
    name: String,
    selected: String,
}
impl Chart {
    pub fn new(name: &ChartsName, selected: bool) -> Self {
        if selected {
            Self {
                name: name.to_string(),
                selected: "selected".to_string(),
            }
        } else {
            Self {
                name: name.to_string(),
                selected: "".to_string(),
            }
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

}

impl From<&(ChartsName, bool)> for Chart {
    fn from((name, selected): &(ChartsName, bool)) -> Self {
        Self::new(name, *selected)
    }
}
