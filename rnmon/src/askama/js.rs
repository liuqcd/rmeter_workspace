use askama::Template;
// use super::Point;
use super::Point;
use super::ToJStr;

#[derive(Template)]
#[template(path = "nmon/echarts.min.js.j2")] 
pub struct Echarts { 
}
impl Echarts {
    pub fn new() -> Self {
        Self {}
    }
}
impl ToJStr for Echarts {
}

#[derive(Template)]
#[template(path = "nmon/options_sys_summ.js.j2")] 
pub struct SystemSum { 
    nmon_id: String,
    nmon_name: String, 
    // series_data_cpu: Vec<Point>,
    series_data_cpu: Vec<Point>,
    // series_data_io: Vec<Point>,
    series_data_io: Vec<Point>,
}

impl SystemSum {
    pub fn new(
        nmon_id: String,
        nmon_name: String,
        // series_data_cpu: Vec<Point>,
        series_data_cpu: Vec<Point>,
        // series_data_io: Vec<Point>,
        series_data_io: Vec<Point>,
    ) -> Self {
        Self {
            nmon_id,
            nmon_name: format!("System Summary {}", nmon_name),
            series_data_cpu,
            series_data_io,
        }
    }
}

impl ToJStr for SystemSum {
}













#[derive(Template)]
#[template(path = "nmon/options_cpu_summ.js.j2")] 
pub struct CpuSumm { 
    nmon_id: String,
    nmon_name: String, 
    axis_label: Vec<String>,
    data_user: Vec<f32>,
    data_sys: Vec<f32>,
    data_wait: Vec<f32>,
}

impl CpuSumm {
    pub fn new(
        nmon_id: String,
        nmon_name: String,
        axis_label: Vec<String>,
        data_user: Vec<f32>,
        data_sys: Vec<f32>,
        data_wait: Vec<f32>,
    ) -> Self {
        Self {
            nmon_id,
            nmon_name: format!("CPU by Processor {}", nmon_name),
            axis_label,
            data_user,
            data_sys,
            data_wait,
        }
    }
}



impl ToJStr for CpuSumm {
}












#[derive(Template)]
#[template(path = "nmon/options_cpu_all.js.j2")] 
pub struct CpuAll { 
    nmon_id: String,
    nmon_name: String, 
    series_data_user: Vec<Point>,
    series_data_sys: Vec<Point>,
    series_data_wait: Vec<Point>,
    series_data_idle: Vec<Point>,
}

impl CpuAll {
    pub fn new(
        nmon_id: String,
        nmon_name: String, 
        series_data_user: Vec<Point>,
        series_data_sys: Vec<Point>,
        series_data_wait: Vec<Point>,
        series_data_idle: Vec<Point>,
    ) -> Self {
        Self {
            nmon_id,
            nmon_name: format!("CPU Total {}", nmon_name),
            series_data_user,
            series_data_sys,
            series_data_wait,
            series_data_idle,
        }
    }
}

impl ToJStr for CpuAll {
}
























// #[derive(Template)]
// #[template(path = "nmon/options_disk_summ_awmn.js.j2")] 
// pub struct DiskSummAwmn { 
//     nmon_id: String,
//     nmon_name: String, 
//     axis_label: Vec<String>,
//     data_avg: Vec<f32>,
//     data_wavg: Vec<f32>,
//     data_max: Vec<f32>,
//     data_min: Vec<f32>,
// }

// impl DiskSummAwmn {
//     pub fn new(
//         nmon_id: String,
//         nmon_name: String,
//         axis_label: Vec<String>,
//         data_avg: Vec<f32>, 
//         data_wavg: Vec<f32>, 
//         data_max: Vec<f32>,
//         data_min: Vec<f32>,
//     ) -> Self {
//         Self {
//             nmon_id,
//             nmon_name: format!("Disk %Busy {}", nmon_name),
//             axis_label,
//             data_avg,
//             data_wavg,
//             data_max,
//             data_min,
//         }
//     }
// }

// impl ToJStr for DiskSummAwmn {
// }






// #[derive(Template)]
// #[template(path = "nmon/options_disk_summ_overtime.js.j2")] 
// pub struct DiskSummOvertime { 
//     nmon_id: String,
//     nmon_name: String, 
//     series_data_read: Vec<Point>,
//     series_data_write: Vec<Point>,
//     series_data_io: Vec<Point>,
// }

// impl DiskSummOvertime {
//     pub fn new(
//         nmon_id: String,
//         nmon_name: String, 
//         series_data_read: Vec<Point>,
//         series_data_write: Vec<Point>,
//         series_data_io: Vec<Point>,
//     ) -> Self {
//         Self {
//             nmon_id,
//             nmon_name: format!("Disk total KB/s Overtime {}", nmon_name),
//             series_data_read,
//             series_data_write,
//             series_data_io,
//         }
//     }
// }

// impl ToJStr for DiskSummOvertime {
// }











#[derive(Template)]
#[template(path = "nmon/options_diskbusy_awmn.js.j2")] 
pub struct DiskBusyAwmn { 
    nmon_id: String,
    nmon_name: String, 
    axis_label: Vec<String>,
    data_avg: Vec<f32>,
    data_wavg: Vec<f32>,
    data_max: Vec<f32>,
    data_min: Vec<f32>,
}

impl DiskBusyAwmn {
    pub fn new(
        nmon_id: String,
        nmon_name: String,
        axis_label: Vec<String>,
        data_avg: Vec<f32>, 
        data_wavg: Vec<f32>, 
        data_max: Vec<f32>,
        data_min: Vec<f32>,
    ) -> Self {
        Self {
            nmon_id,
            nmon_name: format!("Disk %Busy {}", nmon_name),
            axis_label,
            data_avg,
            data_wavg,
            data_max,
            data_min,
        }
    }
}

impl ToJStr for DiskBusyAwmn {
}


















// #[derive(Template)]
// #[template(path = "nmon/options_diskbusy_overtime.js.j2")] 
// pub struct DiskBusyOvertime { 
//     nmon_id: String,
//     nmon_name: String, 
//     series_data: Vec<(String, Vec<Point>)>,
// }


// impl DiskBusyOvertime {
//     pub fn new(
//         nmon_id: String,
//         nmon_name: String,
//         series_data: Vec<(String, Vec<Point>)>,
//     ) -> Self {
//         Self {
//             nmon_id,
//             nmon_name: format!("Disk %Busy over time {}", nmon_name),
//             series_data,
//         }
//     }
// }

// impl ToJStr for DiskBusyOvertime {
// }










// #[derive(Template)]
// #[template(path = "nmon/options_network_io_total.js.j2")] 
// pub struct NetworkIOTotal { 
//     nmon_id: String,
//     nmon_name: String, 
//     series_data_read: Vec<Point>,
//     series_data_write_ve: Vec<Point>,
// }

// impl NetworkIOTotal {
//     pub fn new(
//         nmon_id: String,
//         nmon_name: String, 
//         series_data_read: Vec<Point>,
//         series_data_write_ve: Vec<Point>,
//     ) -> Self {
//         Self {
//             nmon_id,
//             nmon_name: format!("Network I/O Total by Overtime {}", nmon_name),
//             series_data_read,
//             series_data_write_ve,
//         }
//     }
// }

// impl ToJStr for NetworkIOTotal {
// }









// #[derive(Template)]
// #[template(path = "nmon/options_network_io_device.js.j2")] 
// pub struct NetworkIODevice { 
//     nmon_id: String,
//     nmon_name: String, 
//     series_data: Vec<(String, Vec<Point>)>,
// }


// impl NetworkIODevice {
//     pub fn new(
//         nmon_id: String,
//         nmon_name: String,
//         series_data: Vec<(String, Vec<Point>)>,
//     ) -> Self {
//         Self {
//             nmon_id,
//             nmon_name: format!("Network I/O By Device {}", nmon_name),
//             series_data,
//         }
//     }
// }

// impl ToJStr for NetworkIODevice {
// }











#[derive(Template)]
#[template(path = "nmon/options_jfs_file.js.j2")] 
pub struct JfsFile { 
    nmon_id: String,
    nmon_name: String, 
    series_data: Vec<(String, Vec<Point>)>,
}

impl JfsFile {
    pub fn new(
        nmon_id: String,
        nmon_name: String, 
        series_data: Vec<(String, Vec<Point>)>,
    ) -> Self {
        Self {
            nmon_id,
            nmon_name: format!("JFS Filespace Used% {}", nmon_name),
            series_data,
        }
    }
}

impl ToJStr for JfsFile {
}
















#[derive(Template)]
#[template(path = "nmon/options_mem_free.js.j2")] 
pub struct MemFree { 
    nmon_id: String,
    nmon_name: String, 
    series_data_memtotal: Vec<Point>,
    series_data_other: Vec<(String, Vec<Point>)>,
}

impl MemFree {
    pub fn new(
        nmon_id: String,
        nmon_name: String, 
        series_data_memtotal: Vec<Point>,
        series_data_other: Vec<(String, Vec<Point>)>,
    ) -> Self {
        Self {
            nmon_id,
            nmon_name: format!("Memory Free MB {}", nmon_name),
            series_data_memtotal,
            series_data_other,
        }
    }
}

impl ToJStr for MemFree {
}


















#[derive(Template)]
#[template(path = "nmon/options_mem_swap.js.j2")] 
pub struct MemSwap { 
    nmon_id: String,
    nmon_name: String, 
    series_data_memtotal: Vec<Point>,
    series_data_other: Vec<(String, Vec<Point>)>,
}

impl MemSwap {
    pub fn new(
        nmon_id: String,
        nmon_name: String, 
        series_data_memtotal: Vec<Point>,
        series_data_other: Vec<(String, Vec<Point>)>,
    ) -> Self {
        Self {
            nmon_id,
            nmon_name: format!("Memory Swap MB {}", nmon_name),
            series_data_memtotal,
            series_data_other,
        }
    }
}

impl ToJStr for MemSwap {
}









#[derive(Template)]
#[template(path = "nmon/options_mem_active.js.j2")] 
pub struct MemActive { 
    nmon_id: String,
    nmon_name: String, 
    series_data_memtotal: Vec<Point>,
    series_data_other: Vec<(String, Vec<Point>)>,
}

impl MemActive {
    pub fn new(
        nmon_id: String,
        nmon_name: String, 
        series_data_memtotal: Vec<Point>,
        series_data_other: Vec<(String, Vec<Point>)>,
    ) -> Self {
        Self {
            nmon_id,
            nmon_name: format!("Memory Active MB {}", nmon_name),
            series_data_memtotal,
            series_data_other,
        }
    }
}

impl ToJStr for MemActive {
}