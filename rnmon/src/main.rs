pub mod askama;
pub mod output;

use clap::{ArgAction, Parser, arg};
use chrono::{offset::Local};
use log::info;
use log::debug;
use log::trace;
use log::warn;
use std::time::SystemTime;
use anyhow::Ok;
use anyhow::Result;
use anyhow::anyhow;
use std::path::PathBuf;
use std::path::Path;
use std::collections::HashSet;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::str::FromStr;
use chrono::DateTime;
use chrono::FixedOffset;
use ndarray::Array1;
use ndarray::Array2;
use ndarray::ShapeBuilder;
use ndarray_stats::QuantileExt;
use ndarray_stats::SummaryStatisticsExt;

use crate::askama::Point;
use crate::askama::html::Charts;




fn main() -> Result<()>{

    // 处理传入的程序的参数
    let cli_args = Args::parse();

    // log处理
    let mut log = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                chrono::DateTime::<Local>::from(SystemTime::now()).format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                record.target(),
                message
            ))
        })
        .chain(std::io::stdout());

    // log = if let Some(ref logfile) = cli_args.logfile {
    //     log.chain(fern::log_file(logfile)?)
    // } else {
    //     log
    // };

    log = match cli_args.debug {
        0 => log.level(log::LevelFilter::Info),
        1 => log.level(log::LevelFilter::Debug),
        _ => log.level(log::LevelFilter::Trace),
    };

    info!("cli_args: {:?}", &cli_args);
    log.apply()?;


    let nmon_args = cli_args.nmon_args;

    let cell = nmon_args.metrics_name();
    let nmon_datas = nmon_args.nmon_dates(&cell);
    let path = nmon_args.html_output_folder();
    // nmon::nmon(nmon_datas, &cell, path, nmon_args)?;
    nmon(nmon_datas,  path, &nmon_args)?;

    Ok(())
}



/// /// 分析nmon文件，打印各nmon文件的cpu使用，可生成HTML图表（echars.js和html文件）
#[derive(Parser, Debug)]
#[command(author = "liuqxx", version = "0.1.0")]
pub struct Args {
    /// 一个开启DEBUG日志，两个及以上开启trace日志
    #[arg(short, long, action = ArgAction::Count)]
    pub debug: u8,

    #[command(flatten)]
    pub nmon_args: NmonArgs,
}

#[derive(Parser, Debug, Clone)]
pub struct NmonArgs {
    /// 1.当输入为后缀为.nmon的文件时，分析该文件。
    /// 2.当输入为目录时，自动分析该目录下文件后缀为.nmon的文件。
    path: Vec<PathBuf>,
    /// HTML图表输出目录。若该目录已存在，则清除该目录，并重新生成。
    /// 该参数只在nmon子命令时才生效，run子命令时，该参数值与run子命令的outputfolder相同
    #[arg(long, default_value = "./")]
    html_output: Option<PathBuf>,
    #[command(flatten)]
    run_nmon_args: RunNmonArgs
}

impl NmonArgs {
    pub fn new(nmon_path: Vec<PathBuf>, html_output: PathBuf, run_nmon_args: RunNmonArgs) -> Self {
        Self {
            path: nmon_path,
            html_output: Some(html_output),
            run_nmon_args
        }
    }
    pub fn html_output_folder(&self) -> PathBuf {
        self.html_output.clone().unwrap()
    }
    pub fn metrics_name(&self) -> HashSet<String> {
        // 设置要从nmon文件在处理的指标名称
        // 第四版要求 CPUXX、CPU_ALL、DISKXFER、DISKBUSY、DISKREAD、DISKWRITE、DISKXFER、DISKBSIZE必填输入
        let mut cell = HashSet::new();
        if self.run_nmon_args.cpu || self.run_nmon_args.html {
            for i in 0..10 {
                cell.insert(format!("CPU{}", i));
            }
            for i in 0..100 {
                cell.insert(format!("CPU{:02}", i));
            }
            for i in 0..1000 {
                cell.insert(format!("CPU{:03}", i));
            }
            cell.insert("CPU_ALL".into());
            // nmon system_summary
            cell.insert("DISKXFER".into());
        }
        if self.run_nmon_args.disk_io {
            cell.insert("DISKXFER".into());
        }
        if self.run_nmon_args.disk_summary {
            cell.insert("DISKREAD".into());
            cell.insert("DISKWRITE".into());
            cell.insert("DISKBSIZE".into());
        }
        if self.run_nmon_args.disk_busy {
            cell.insert("DISKBUSY".into());
        }
        if self.run_nmon_args.mem_free || self.run_nmon_args.mem_active || self.run_nmon_args.mem_active {
            cell.insert("MEM".into());
        }
        if self.run_nmon_args.jfsfile {
            cell.insert("JFSFILE".into());
        }
        if self.run_nmon_args.disk_io {
            cell.insert("DISKXFER".into());
        }
        if self.run_nmon_args.disk_summary {
            cell.insert("DISKXFER".into());
        }
        if self.run_nmon_args.net {
            cell.insert("NET".into());
            // cell.insert("NETPACKET");
        }

        cell
    }

    pub fn nmon_dates(&self, cell: &HashSet<String>) -> Vec<NmonData> {
        let mut files = Vec::new();
        self.path.iter().for_each(|p| {
            if p.is_file() {
                files.push(p.to_path_buf());
            } else if p.is_dir() {
                for entry in std::fs::read_dir(p).unwrap() {
                    let entry = entry.unwrap();
                    let path = entry.path();
                    if path.is_file() {
                        let name = path.file_name().unwrap();
                        let name = name.to_string_lossy();
                        if name.ends_with(".nmon") {
                            files.push(path);
                        }
                    }
                }
            } else {
                panic!("输入参数即不是目录也不是文件");
            }
        });
        files.sort();
        let mut res = Vec::new();
        files.iter().for_each(|p| {
            res.push(NmonData::new(p, &cell).unwrap());
        });
        res
    }
}


#[derive(Parser, Debug, Clone)]
pub struct RunNmonArgs {
    /// 对输入的nmon文件进行分析，并生成HTML图表，当指定html时cpu默认为true
    #[arg(long, action = ArgAction::SetTrue)]
    pub html: bool,

    /// 分析nmon文件时，设置对cpu使用进行分析
    #[arg(long, default_value_t = true)]
    pub cpu: bool,

    /// 分析nmon文件时，设置对mem使用进行分析
    #[arg(long, action = ArgAction::SetTrue)]
    pub mem_free: bool,

    /// 分析nmon文件时，设置对mem使用进行分析
    #[arg(long, action = ArgAction::SetTrue)]
    pub mem_active: bool,

    /// 分析nmon文件时，设置对mem使用进行分析
    #[arg(long, action = ArgAction::SetTrue)]
    pub mem_swap: bool,

    /// 分析nmon文件时，设置对jfsfile使用进行分析
    #[arg(long, action = ArgAction::SetTrue)]
    pub jfsfile: bool,

    /// 分析nmon文件时，设置对disk_busy使用进行分析
    #[arg(long, action = ArgAction::SetTrue)]
    pub disk_busy: bool,

    /// 分析nmon文件时，设置对DISKBUSY使用进行分析，暂不支持todo
    #[arg(long, action = ArgAction::SetTrue)]
    pub disk_io: bool,

    /// 分析nmon文件时，设置对DISK_SUMM使用进行分析，暂不支持todo
    #[arg(long, action = ArgAction::SetTrue)]
    pub disk_summary: bool,

    /// 分析nmon文件时，设置对net使用进行分析，不支持todo
    #[arg(long, action = ArgAction::SetTrue)]
    pub net: bool,
}


pub fn nmon(
    nmon_datas: Vec<NmonData>,
    // cell: &HashSet<String>,
    path: PathBuf,
    nmonargs: &NmonArgs,
) -> Result<()> {
    let mut res_nmon_txt = String::new();
    let mut js = String::new();
    let mut html_template_data = Vec::new();
    let mut charts = Vec::new();

    // 生成HTML图表
    if nmonargs.run_nmon_args.html {
        if nmonargs.run_nmon_args.cpu || nmonargs.run_nmon_args.html {
            charts.push(Charts::new("SYS_SUMM".to_string(), "selected".to_string()));
            charts.push(Charts::new("CPU_SUMM".to_string(), "".to_string()));
            charts.push(Charts::new("CPU_ALL".to_string(), "".to_string()));
        }
        if nmonargs.run_nmon_args.jfsfile {
            charts.push(Charts::new("JFSFILE".to_string(), "".to_string()));
        }
        if nmonargs.run_nmon_args.mem_free {
            charts.push(Charts::new("MEM_FREE".to_string(), "".to_string()));
        }
        if nmonargs.run_nmon_args.mem_active {
            charts.push(Charts::new("MEM_ACTIVE".to_string(), "".to_string()));
        }
        if nmonargs.run_nmon_args.mem_swap {
            charts.push(Charts::new("MEM_SWAP".to_string(), "".to_string()));
        }
        if nmonargs.run_nmon_args.disk_busy {
            charts.push(Charts::new("DISKBUSY_AWMN".to_string(), "".to_string()));
        }
    }

    for (i, ndata) in nmon_datas.iter().enumerate() {
        // 使用表格形式，打印CPU_ALL到屏蔽，包括： mean, stv
        let res = output::console_print_cpuall(ndata);
        res_nmon_txt.push_str(res.as_str());
        res_nmon_txt.push_str("\n");

        // 生成js
        if nmonargs.run_nmon_args.html {
            let id = format!("{:03}", i);
            html_template_data.push((id.clone(), ndata.filename().to_string()));

            if nmonargs.run_nmon_args.cpu || nmonargs.run_nmon_args.html {
                let cpu_all = askama::js_cpu_all(&id, ndata);
                js.push_str(&cpu_all);
                js.push('\n');

                let system_summary = askama::js_system_summary(&id, ndata);
                js.push_str(&system_summary);
                js.push('\n');

                let cpu_summ = askama::js_cpu_summ(&id, ndata);
                js.push_str(&cpu_summ);
                js.push('\n');
            }

            if nmonargs.run_nmon_args.jfsfile {
                let jfs_file = askama::js_jfs_file(&id, ndata);
                js.push_str(&jfs_file);
                js.push('\n');
            }

            if nmonargs.run_nmon_args.mem_free {
                let mem_free = askama::js_mem_free(&id, ndata);
                js.push_str(&mem_free);
                js.push('\n');
            }
            if nmonargs.run_nmon_args.mem_active {
                let mem_active = askama::js_mem_active(&id, ndata);
                js.push_str(&mem_active);
                js.push('\n');
            }
            if nmonargs.run_nmon_args.mem_swap {
                let mem_swap = askama::js_mem_swap(&id, ndata);
                js.push_str(&mem_swap);
                js.push('\n');
            }

            if nmonargs.run_nmon_args.disk_busy {
                let disk_busy = askama::js_disk_busy_awmn(&id, ndata);
                // println!("1111111111111: {}", disk_busy);
                js.push_str(&disk_busy);
                js.push('\n');
            }
        }
    }
    // 保存console输出到文件
    output::save(path.join("res.nmon.txt"), res_nmon_txt);

    if nmonargs.run_nmon_args.html {
        // 保存图表数据到js文件
        output::save(path.join("index_nmons_data.js"), js);
        // 生成echarts.min.js
        let echarts = askama::js_echarts();
        output::save(path.join("echarts.min.js"), echarts);

        // 保存html文件
        let html = askama::html(html_template_data, charts);
        output::save(path.join("index_nmons.html"), html);
    }

    Ok(())
}











pub struct NmonData {
    filename: String,
    measurements: BTreeMap<String, Measurement>,
}

impl NmonData {
    pub fn filename(&self) -> &str {
        &self.filename
    }
    pub fn measurement(&self, name: &str) -> Option<&Measurement> {
        self.measurements.get(name)
    }
    pub fn measurements(&self) -> &BTreeMap<String, Measurement> {
        &self.measurements
    }

    pub fn new(path: &Path, cell: &HashSet<String>) -> Result<Self> {
        debug!("打算收集的指标：{:?}", cell);

        let filename = path.file_name().unwrap().to_string_lossy().to_string();
        let f = File::open(path).map_err(|e| anyhow!("{}文件打开失败：{}", &filename, e))?;
        let br = BufReader::new(f);
        let mut line_iter = br.lines().map(|l| l.unwrap());

        // 解析nmon到内存
        let mut headers = BTreeMap::new(); // csv headers
        let mut notes = BTreeMap::new(); // 各指标数据行的第二列
        let mut data = BTreeMap::new(); // 各指标数据行的[2..]
        let mut lens = BTreeMap::new(); // 各指标数据行的行数
        let mut zzzz = Vec::new(); // 图表X轴的时间序列, ZZZZ,T0120,17:42:21,16-JUN-2020

        while let Some(line) = line_iter.next() {
            let vec: Vec<&str> = line.split(',').collect();
            let name = vec[0];
            trace!("[{}]文件, 处理数据行: {}", filename, line);
            trace!("[{}]文件, 处理数据行转换为数组，其长度为：{}，数组为：{:?}", filename, vec.len(), vec);
            // 判定是否为时间序列
            // ZZZZ,T0001,17:40:19,16-JUN-2020
            if name == "ZZZZ" {
                let time_str = vec[2];
                let date_str = vec[3];
                let s = format!("{}T{} +0000", date_str, time_str);
                let fmt = "%d-%b-%YT%H:%M:%S %z";
                let ndt = DateTime::parse_from_str(&s, fmt).map_err(|e| anyhow!("{}文件，数据解析ZZZZ时间序列出错: {}, s: '{}' 和 fmt: '{}' 解析格式不匹配，原始数据vec: {:?}", &filename, e, &s, fmt, vec))?;
                zzzz.push(ndt);
                trace!("[{}]文件, 收集时间戳到ZZZZ数据: {:?}", filename, ndt);
            }
            // 判定是否为要收集的指标
            if cell.contains(name) {
                trace!("[{}]文件, 开始处理要收集要的指标数据", filename);
                if !headers.contains_key(name) {
                    let mut header: Vec<String> = vec[2..].iter().map(|e| e.to_string()).collect();
                    let mut note = vec[1].to_string();
                    trace!("[{}]文件, [{}]指标数据header头转换为数组: {:?}", filename, name, header);
                    // VM	"Paging and Virtual Memory"	nr_dirty	nr_writeback	nr_unstable	nr_page_table_pages	nr_mapped	nr_slab_reclaimable	pgpgin	pgpgout	pswpin	pswpout	pgfree	pgactivate	pgdeactivate	pgfault	pgmajfault	pginodesteal	slabs_scanned	kswapd_steal	kswapd_inodesteal	pageoutrun	allocstall	pgrotated	pgalloc_high	pgalloc_normal	pgalloc_dma	pgrefill_high	pgrefill_normal	pgrefill_dma	pgsteal_high	pgsteal_normal	pgsteal_dma	pgscan_kswapd_high	pgscan_kswapd_normal	pgscan_kswapd_dma	pgscan_direct_high	pgscan_direct_normal	pgscan_direct_dma
                    // VM	T0001	27	0	0	6688	59502	23700	1040	80	0	0	8848	84	0	12898	8	0	0	0	0	0	0	0	0	8711	0	0	0	0	0	0	0	0	0	0	0	0	0
                    // ...
                    // VM	T0001	"Paging and Virtual Memory"	nr_dirty	nr_writeback	nr_unstable	nr_page_table_pages	nr_mapped	nr_slab	pgpgin	pgpgout	pswpin	pswpout	pgfree	pgactivate	pgdeactivate	pgfault	pgmajfault	pginodesteal	slabs_scanned	kswapd_steal	kswapd_inodesteal	pageoutrun	allocstall	pgrotated	pgalloc_high	pgalloc_normal	pgalloc_dma	pgrefill_high	pgrefill_normal	pgrefill_dma	pgsteal_high	pgsteal_normal	pgsteal_dma	pgscan_kswapd_high	pgscan_kswapd_normal	pgscan_kswapd_dma	pgscan_direct_high	pgscan_direct_normal	pgscan_direct_dma
                    // VM	T0001	-1	-1	-1	8077	-1	-1	-1	-1	-1	-1	-1	-1	-1	-1	-1	5800	4169	-1	-1	0	32	-1	7475286	-1	-1	-1	-1	-1	-1	0	544100	-1	81	820	0	-1	-1
                    if name == "VM" {
                        // 当指标列名note以"T"开头时，则列名长度比数据列长度多1，故列名数据减1
                        if note.starts_with("T") {
                            header = vec[3..].iter().map(|e| e.to_string()).collect();
                            note = vec[2].to_string();
                            trace!("[{}]文件, [VM]指标数据header头特殊处理，转换为数组: {:?}", filename, header);
                        }
                    }
                    headers.insert(name.to_string(), header);
                    notes.insert(name.to_string(), note);
                    data.insert(name.to_string(), Vec::new());
                    lens.insert(name.to_string(), 0 as usize);
                } else {
                    // 数据列名的长度是否相等
                    let name_len = headers.get(name).unwrap().len();
                    trace!("[{}]文件, [{}]指标数据，列名长度：{}", filename, name, name_len);
                    // 增加数据
                    let value = data.get_mut(name).unwrap();
                    trace!("[{}]文件, [{}]指标数据，已增加到data数组里的长度为：{}", filename, name, value.len());
                    // 情况1：
                    // net 的数据列有可能在运行时多出几列，按列名长度，忽略多出的列
                    // 处理方式： vec[2..name_len + 2], 直接截断多余的数据
                    //
                    // 情况2:
                    // range out index 60 out of range for slice of length 53
                    // 当测试突然停止，从服务器取得nmon文件时，可能服务器正在向nmon文件中向数据，此时取nmon文件可能导致指标数据没写完，导致数据操作失败
                    // 处理方式：数据长度大于等于列名长度才收集该数据，否则抛弃该数据
                    let data_len = vec[2..].len();
                    if data_len < name_len {
                        warn!("[{}]文件, [{}]指标数据，指标数据长度[{}]小于列名长度[{}]，数据不完成，抛弃它：{}", filename, name, data_len, name_len, line);
                    }else {
                        vec[2..name_len + 2]
                            .iter()
                            .map(|e| {
                                if e.is_empty() {
                                    0.0
                                } else {
                                    f32::from_str(e).expect(&format!(
                                        "filename: {}, line: {:?}\n, e: {}, ",
                                        &filename, line, e,
                                    ))
                                }
                            })
                            .for_each(|e| value.push(e));

                        // 指标长度加一，方便下面检查数据
                        let l = lens.get_mut(name).unwrap();
                        *l = *l + 1;
                        trace!("[{}]文件, [{}]指标数据，已收集的数据长度加1后值为：{}", filename, name, l);
                    }
                }
            }
        }

        // 各指标数据长度以及ZZZZ长度有可能不一致，取最小化长度，忽略多出的数据
        let mut vec_len: Vec<usize> = lens.values().map(|l| *l).collect();
        vec_len.push(zzzz.len());
        let min = *vec_len.iter().min().unwrap();
        if min < zzzz.len() {
            warn!(
                "[{}]文件，数据解析ZZZZ时间序列长度{} > 指标数据最小行数{}, 以最小行数截取ZZZZ时间序列数据",
                filename,
                zzzz.len(),
                min
            );
            zzzz.truncate(min);
        }
        for (key, vec) in data.iter_mut() {
            let colsize = headers.get(key).unwrap().len();
            let rowsize = *lens.get(key).unwrap();
            if min < rowsize {
                warn!(
                    "[{}]文件，[{}]指标数据行数{} > 指标数据行数最小值orZZZZ时间序列数据行数{}, 以最小行数截取指标数据",
                    filename, key, rowsize, min
                );
                vec.truncate(min * colsize);
            }
        }

        // 数据转换为ndarray里的Array2结构
        // let mut ndata: BTreeMap<String, _> = BTreeMap::new();
        // let mut ndata: BTreeMap<String, ndarray::ArrayBase<ndarray::OwnedRepr<f32>, ndarray::Ix2>> = BTreeMap::new();
        let rowsize = min;
        // let mut ndata = BTreeMap::new();
        let mut measurements = BTreeMap::new();
        for (name, vec) in data.into_iter() {
            let header = headers.get(&name).unwrap();
            let colsize = header.len();
            let note = notes.get(&name).unwrap();
            let measurement = Measurement::new(
                &filename,
                &name,
                note,
                header.clone(),
                &zzzz,
                vec,
                (rowsize, colsize),
            );
            measurements.insert(name.to_string(), measurement);
        }

        Ok(Self {
            filename,
            measurements,
        })
    }
}


pub struct Measurement {
    filename: String,
    name: String,
    note: String,
    header: Vec<String>,
    zzzz: Vec<DateTime<FixedOffset>>,
    // zzzz: Array1<DateTime<FixedOffset>>,
    data: Array2<f32>,
}

impl Measurement {
    pub fn new(
        filename: &str,
        name: &str,
        note: &str,
        header: Vec<String>,
        zzzz: &Vec<DateTime<FixedOffset>>,
        // zzzz: Array1<DateTime<FixedOffset>>,
        data: Vec<f32>,
        (rowsize, colsize): (usize, usize),
    ) -> Self {
        let array2 =
            Array2::from_shape_vec((rowsize, colsize).set_f(false), data).expect(&format!(
                "{}文件，{}指标从vec转化为Array2({}, {})失败",
                &filename, name, rowsize, colsize
            ));
        Self {
            filename: filename.to_string(),
            name: name.to_string(),
            note: note.to_string(),
            header,
            zzzz: zzzz.clone(),
            data: array2,
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn filename(&self) -> &str {
        &self.filename
    }
    pub fn note(&self) -> &str {
        &self.note
    }
    pub fn header(&self) -> &Vec<String> {
        &self.header
    }
    pub fn zzzz(&self) -> Vec<DateTime<FixedOffset>> {
        self.zzzz.clone()
    }
    pub fn column_echartjs_vec(&self, columns: &[&str]) -> Vec<(String, Vec<Point>)> {
        let mut vec = Vec::new();
        for name in columns {
            if let Some(echarsjs_data) = self.column_echartjs_overtime(name) {
                vec.push((name.to_string(), echarsjs_data));
            }
        }
        vec
    }
    // pub fn columns_echarsjs_awmm(&self) -> Vec<(String, Vec<Point>)> {
    //     let mut vec = Vec::new();
    //     for name in self.columns_echartjs_overtime() {
    //         if let Some(echarsjs_data) = self.column_echartjs_overtime(&name) {
    //             vec.push((name, echarsjs_data));
    //         }
    //     }
    //     vec
    // }

    // pub fn xalis_datetime_range(&self) -> Range<DateTime<FixedOffset>> {
    //     let len = self.zzzz.len();
    //     // 17:40:19,16-JUN-2020
    //     // let start = self.zzzz[0].clone();
    //     let start = self.zzzz[0];
    //     //17:42:21,16-JUN-2020
    //     // let xend = DateTime::parse_from_str("16-JUN-2020T17:42:28 +0000", "%d-%b-%YT%H:%M:%S %z").unwrap();
    //     let end = self.zzzz[len - 1] + Duration::from_secs(1);
    //     start..end
    // }

    fn idx_of_column(&self, name: &str) -> Option<usize> {
        self.header
            .iter()
            .enumerate()
            .find(|(_idx, e)| **e == name)
            .map(|(idx, _)| idx)
    }

    pub fn column(&self, name: &str) -> Option<Array1<f32>> {
        if let Some(idx) = self.idx_of_column(name) {
            let array1 = self.data.column(idx).map(|x| x.to_owned());
            Some(array1)
        } else {
            None
        }
    }
    /// 某列所有数据求均值
    pub fn column_mean(&self, name: &str) -> Option<f32> {
        if let Some(array1) = self.column(name) {
            array1.mean()
        } else {
            None
        }
    }
    pub fn column_echartjs_overtime(&self, name: &str) -> Option<Vec<Point>> {
        if let Some(idx) = self.idx_of_column(name) {
            let array1 = self.data.column(idx);
            let col_data: Vec<Point> = self
                .zzzz
                .iter()
                .zip(array1.iter())
                .map(|(x, y)| Point::new(format!("{}", x.format("%Y-%m-%d %H:%M:%S")), *y))
                .collect();
            Some(col_data)
        } else {
            None
        }
    }

    pub fn rows_max(&self) -> Vec<f32> {
        self.data
            .columns()
            .into_iter()
            .map(|col| *col.max().unwrap())
            .collect()
    }
    pub fn rows_min(&self) -> Vec<f32> {
        self.data
            .columns()
            .into_iter()
            .map(|col| *col.min().unwrap())
            .collect()
    }
    /// 按时间序列把每行的数据后，再取平均
    pub fn rows_mean(&self) -> Vec<f32> {
        self.data
            .columns()
            .into_iter()
            .map(|col| col.mean().unwrap())
            .collect()
    }
    /// 按时间序列，求取标准：
    pub fn rows_stdev(&self) -> Vec<f32> {
        self.data
            .columns()
            .into_iter()
            .map(|col| col.std(0.))
            .collect()
    }
    /// 按时间序列，求取wavg：
    pub fn rows_wavg(&self) -> Vec<f32> {
        let vec: Vec<f32> = self
            .data
            .columns()
            .into_iter()
            .map(|col| {
                let mean = col.mean().unwrap();
                if mean == 0. {
                    0.
                } else {
                    col.weighted_mean(&col).unwrap()
                }
            })
            .collect();
        vec
    }
    // pub fn rows_sum_iter(&self) -> impl Iterator<Item = f32> {
    //     self.data.rows().into_iter().map(|column| column.sum())
    // }

    pub fn columns_echartjs_overtime(&self) -> Vec<(String, Vec<Point>)> {
        self.header
            .iter()
            .zip(self.data.columns())
            .map(|(header, column)| {
                let vec: Vec<Point> = self
                    .zzzz
                    .iter()
                    .zip(column.iter())
                    .map(|(x, y)| Point::new(format!("{}", x.format("%Y-%m-%d %H:%M:%S")), *y))
                    .collect();
                (header.clone(), vec)
            })
            .collect()
    }
    pub fn column_sum_echartjs_overtime(&self) -> Vec<Point> {
        let array1: Array1<f32> = self
            .data
            .rows()
            .into_iter()
            .map(|column| column.sum())
            .collect();
        let col_data: Vec<Point> = self
            .zzzz
            .iter()
            .zip(array1.iter())
            .map(|(x, y)| Point::new(format!("{}", x.format("%Y-%m-%d %H:%M:%S")), *y))
            .collect();
        col_data
    }
}
