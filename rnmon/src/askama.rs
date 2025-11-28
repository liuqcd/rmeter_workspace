
pub mod html;
pub mod js;

use ndarray::Array1;



use askama::Template;

use std::collections::BTreeMap;
pub use html::NmonHtmlTemplate;
pub use html::NmonFile;
pub use html::Chart;

use super::Measurement;

use super::NmonData;

pub fn js_echarts() -> String {
    let echarts = js::Echarts::new();
    echarts.to_js_str()

}


#[derive(Clone, Debug)]
// pub struct TimeSeriesData {
pub struct Point {
    x: String,
    y: f32,
}

// impl TimeSeriesData {
impl Point {
    pub fn new(x: String, y: f32) -> Self {
        Self {
            x,
            y,
        }
    }
}

trait ToJStr: Template {
    fn to_js_str(&self) -> String {
        self.render().unwrap()
    }
}

// pub fn html(template_data: Vec<(String, String)>) -> String {
pub fn html(template_data: Vec<(String, String)>, charts: Vec<Chart>) -> String {

    let mut nmonfiles = Vec::new();
    for (id, name) in template_data.into_iter() {
        nmonfiles.push(
            NmonFile::new(id, name, "selected".to_string())
        )
    }

    let html = NmonHtmlTemplate::new(nmonfiles, charts);

    html.render().unwrap()
}







pub fn js_system_summary(data: &NmonData) -> String {

    let cpu_all = data.measurement("CPU_ALL").unwrap();
    let cpu_idle = cpu_all.column("Idle%").unwrap();
    let cpu_used = Array1::from_vec(vec![100.0; cpu_idle.len()]) - cpu_idle;
    // let zzzz = data.xalis_datetime_to_own();
    let zzzz = cpu_all.zzzz();
    let series_data_cpu: Vec<Point> = zzzz.iter().zip(cpu_used.iter())
        .map(|(x, y)|
            Point::new(
                format!("{}", x.format("%Y-%m-%d %H:%M:%S")),
                *y
            )
        ).collect();

    let disk_xfer = data.measurement("DISKXFER").unwrap();
    let series_data_io = disk_xfer.column_sum_echartjs_overtime();

    let b = js::SystemSum::new(
        data.filename().to_string(),
        series_data_cpu,
        series_data_io
    );
    b.to_js_str()
}














pub fn js_cpu_summ(data: &NmonData) -> String {
    let measurements = data.measurements();
    let cpuxx: BTreeMap<&String, &Measurement> = measurements.iter().filter(|(k,_v)| k.starts_with("CPU") && !k.starts_with("CPU_ALL")).collect();

    let mut axis_label = Vec::new();
    let mut data_user = Vec::new();
    let mut data_sys = Vec::new();
    let mut data_wait = Vec::new();

    for (name, measurement) in cpuxx.iter() {
        axis_label.push(name.to_string());
        let user = measurement.column_mean("User%").unwrap();
        let sys = measurement.column_mean("Sys%").unwrap();
        let wait = measurement.column_mean("Wait%").unwrap();
        data_user.push(user);
        data_sys.push(sys);
        data_wait.push(wait);
    }

    let b = js::CpuSumm::new(
        data.filename().to_string(),
        axis_label,
        data_user,
        data_sys,
        data_wait,
    );
    b.to_js_str()
}










pub fn js_cpu_all(data: &NmonData) -> String {
    let cpu_all = data.measurement("CPU_ALL").unwrap();
    let series_data_user = cpu_all.column_echartjs_overtime("User%").unwrap();
    let series_data_sys = cpu_all.column_echartjs_overtime("Sys%").unwrap();
    let series_data_wait = cpu_all.column_echartjs_overtime("Wait%").unwrap();
    let series_data_idle = cpu_all.column_echartjs_overtime("Idle%").unwrap();

    let b = js::CpuAll::new(
        data.filename().to_string(),
        series_data_user,
        series_data_sys,
        series_data_wait,
        series_data_idle,
    );

    b.to_js_str()
}


pub fn js_jfsfile(data: &NmonData) -> String {
    let measurement = data.measurement("JFSFILE").unwrap();
    let series_data = measurement.columns_echartjs_overtime();

    let b = js::JfsFile::new(
        data.filename().to_string(),
        series_data,
    );

    b.to_js_str()
}




pub fn js_mem_free(data: &NmonData) -> String {
    let measurement = data.measurement("MEM").unwrap();

    let series_data_total = measurement.column_echartjs_overtime("memtotal").unwrap();
    let series_data_other = measurement.column_echartjs_vec(&["memfree", "cached", "buffers"]);

    let b = js::MemFree::new(
        data.filename().to_string(),
        series_data_total,
        series_data_other,
    );

    b.to_js_str()
}






pub fn js_mem_swap(data: &NmonData) -> String {
    let measurement = data.measurement("MEM").unwrap();

    let series_data_total = measurement.column_echartjs_overtime("swaptotal").unwrap();
    let series_data_other = measurement.column_echartjs_vec(&["swapfree", "swapcached",]);

    let b = js::MemSwap::new(
        data.filename().to_string(),
        series_data_total,
        series_data_other,
    );

    b.to_js_str()
}











pub fn js_mem_active(data: &NmonData) -> String {
    let measurement = data.measurement("MEM").unwrap();

    let series_data_total = measurement.column_echartjs_overtime("memtotal").unwrap();
    let series_data_other = measurement.column_echartjs_vec(&["active", "inactive",]);

    let b = js::MemActive::new(
        data.filename().to_string(),
        series_data_total,
        series_data_other,
    );

    b.to_js_str()
}











pub fn js_diskbusy_awmn(data: &NmonData) -> String {
    let measurement = data.measurement("DISKBUSY").unwrap();

    let axis_label = measurement.header().clone();
    let data_avg = measurement.rows_mean();
    // echarts.js 堆叠柱状图时值不叠加
    let mut data_wavg= measurement.rows_wavg();
    data_wavg = data_wavg.iter().zip(data_avg.iter()).map(|(wavg, avg)| *wavg - *avg).collect();

    let data_max = measurement.rows_max();
    let data_min = measurement.rows_min();

    let b = js::DiskBusyAwmn::new(
        data.filename().to_string(),
        axis_label,
        data_avg,
        data_wavg,
        data_max,
        data_min,
    );
    b.to_js_str()
}
