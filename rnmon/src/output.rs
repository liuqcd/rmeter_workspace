
use crate::NmonData;


use std::fmt::Display;
use tabled::{
    settings::Style,
};
use std::path::PathBuf;

use std::io::Write;
use std::fs::OpenOptions;
pub fn save(path: PathBuf , data: String ) {
    let mut f = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(path.as_path())
                    .unwrap();

    f.write_all(data.as_bytes()).unwrap();
}



pub fn console_print_cpuall(nmon: &NmonData) -> String {
    let cpu_all = nmon.measurement("CPU_ALL").unwrap();
    let filename = cpu_all.filename();
    let mean = cpu_all.rows_mean();
    let stdev = cpu_all.rows_stdev();

    let mut res_nmon_text = String::new();
    let measurement_name = cpu_all.name();
    let header = cpu_all.header();
    let pheader = PrintData::new("FILENAME", "KEY", "NOTE", "TAG", header);
    let note = cpu_all.note();
    let pavg = PrintData::new(filename, measurement_name, note, "mean", mean);
    let pstdev = PrintData::new(filename, measurement_name, note, "stdev", stdev);
    let mut builder = tabled::builder::Builder::default();
    builder.push_record(pheader);
    builder.push_record(pavg);
    builder.push_record(pstdev);
    let table = builder.build().with(Style::rounded()).to_string();
    println!("{}", table);
    res_nmon_text.push_str(table.as_str());

    res_nmon_text
}

#[derive(Debug, Default)]
struct PrintData<T>
where
    T: std::iter::IntoIterator,
    T::Item: Display,
{
    filename: String,
    _key: String,
    note: String,
    tag: String,
    values: T,
}
impl<T> PrintData<T>
where
    T: std::iter::IntoIterator,
    T::Item: Display,
{
    fn new(filename: &str, key: &str, note: &str, tag: &str, values: T) -> Self {
        PrintData {
            filename: filename.to_string(),
            _key: key.to_string(),
            note: note.to_string(),
            tag: tag.to_string(),
            values,
        }
    }
}
impl<T> std::iter::IntoIterator for PrintData<T>
where
    T: std::iter::IntoIterator,
    T::Item: Display,
{
    type Item = String;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        let mut vec = Vec::new();
        vec.push(self.filename);
        vec.push(self.note);
        vec.push(self.tag);
        self.values
            .into_iter()
            .for_each(|e| vec.push(e.to_string()));
        vec.into_iter()
    }
}
