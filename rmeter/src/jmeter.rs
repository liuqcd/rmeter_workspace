use crate::client::JMeterArgs;
use anyhow::Result;
use anyhow::anyhow;
use std::path::PathBuf;
use std::path::Path;
use std::time::Duration;
use std::fs::File;
use std::io::BufReader;
use log::error;
use log::info;
use log::debug;
use log::warn;


pub struct JMeter {
    args: JMeterArgs,
}

// impl Run for JMeter {}

impl JMeter {
    pub fn new(args: JMeterArgs) -> Self {
        JMeter {
            args,
        }
    }

    pub fn get_tmp_dir(&self) -> PathBuf {
        self.args.tmpdir.clone()
    }
    fn overall_granularity_params(&self) -> Vec<String> {
        let monitor_duration = Duration::from_secs(self.args.rampup)
            + Duration::from_secs(self.args.duration)
            + Duration::from_secs(3);
        debug!(
            "动态计算，增加3秒jmeter启动时间后，monitor_duration: {}secs",
            monitor_duration.as_secs()
        );
        let monitor_interval = Duration::from_secs(monitor_duration.div_f32(1440.0).as_secs() + 1);
        debug!("动态计算，monitor_interval: {}", monitor_interval.as_secs());
        let monitor_count = monitor_duration.div_duration_f32(monitor_interval) as u64;
        debug!("动态计算，monitor_count: {}", monitor_count);
        let overall_granularity_duration = match monitor_duration.as_secs() {
            // <4min, 最大240个点
            0..241 => Duration::from_secs(1),
            // <31min, 最大930个点
            241..1861 => Duration::from_secs(2),
            // <2.5h, 最大900个点
            1861..9001 => Duration::from_secs(10),
            // <5.5h, 最大990个点
            9001..19801 => Duration::from_secs(20),
            // <12.5h, 最大1500个点
            19801..45001 => Duration::from_secs(30),
            // 24h, 最大1440个点
            // 13h, 最大780个点
            _ => Duration::from_secs(60),
        };
        let ms = overall_granularity_duration.as_secs() * 1000;
        debug!(
            "动态计算: -Jjmeter.reportgenerator.overall_granularity={}",
            ms
        );
        let mut vec = Vec::new();
        vec.push(format!("-J"));
        vec.push(format!("jmeter.reportgenerator.overall_granularity={}", ms));
        vec
    }

    fn propfile_params(&self) -> Result<Vec<String>> {
        let mut params = Vec::new();
        if let Some(propfile) = &self.args.propfile {
            if propfile.try_exists().map_err(|e| anyhow!("{}不存在: {}", propfile.display(), e))? {
                let contents = std::fs::read_to_string(propfile)
                    .map_err(|e| anyhow!("读取测试配置文件{}失败: {}", propfile.display(), e))?;
                contents
                    .lines()
                    .map(|line| line.trim())
                    .filter(|line| !line.starts_with("#") && !line.is_empty())
                    .for_each(|line| {
                        if line.contains("=") {
                            let (key, value) = line.split_once("=").unwrap();
                            params.push(format!("-J"));
                            params.push(format!("{}={}", key, value));
                        } else {
                            params.push(format!("-J"));
                            params.push(format!("{}", line));
                        }
                    });
            }else {
                params.push(format!("-J"));
                params.push(format!("{}", "server.rmi.ssl.disable=false"));
                params.push(format!("-J"));
                params.push(format!("{}", "summariser.interval=10"));
                params.push(format!("-J"));
                params.push(format!("{}", "jmeter.save.saveservice.timestamp_format=yyyyMMdd-HHmmss.SSS"));
                params.push(format!("-J"));
                params.push(format!("{}", "server.rmi.ssl.disable=false"));

                debug!("propfile参数使用默认值");
            }
        }
        debug!("propfile参数: {:?}", params);
        Ok(params)
    }

    fn jmeterproperty_params(&self) -> Vec<String> {
        let mut params = Vec::new();
        if let Some(ref vec) = self.args.jmeterproperty {
            vec.iter().for_each(|p| {
                params.push(format!("-J"));
                params.push(format!("{}", p));
            });
        }
        params
    }

    fn thread_group_params(&self) -> Vec<String> {
        let mut params = Vec::new();
        params.push(format!("-J"));
        params.push(format!("Threads={}", self.args.thread_num));
        params.push(format!("-J"));
        params.push(format!("LoopOrRampupCount={}", self.args.count));
        params.push(format!("-J"));
        params.push(format!("Duration={}", self.args.duration));
        params.push(format!("-J"));
        params.push(format!("Rampup={}", self.args.rampup));
        params.push(format!("-J"));
        params.push(format!("Threads={}", self.args.thread_num));
        params.push(format!("-J"));
        params.push(format!("LoopOrRampupCount={}", self.args.count));
        params.push(format!("-J"));
        params.push(format!("Duration={}", self.args.duration));
        params.push(format!("-J"));
        params.push(format!("Rampup={}", self.args.rampup));
        debug!("线程组设置参数为: {:?}", params);
        params
    }

    fn jmxfile_params(&self) -> Result<Vec<String>> {
        if std::path::Path::try_exists(&self.args.jmxfile)? {
            let mut params = Vec::new();
            params.push(String::from("-n"));
            params.push(String::from("-t"));
            params.push(format!("{}", self.args.jmxfile.display()));
            params.push(String::from("-l"));
            // params.push(String::from("res.jtl"));
            params.push(format!("{}/res.jtl", self.args.tmpdir.display()));
            params.push(String::from("-j"));
            params.push(format!("{}/jmeter.log", self.args.tmpdir.display()));
            params.push(String::from("-e"));
            params.push(String::from("-o"));
            params.push(format!("{}/res", self.args.tmpdir.display()));
            debug!("jmxfile参数: {:?}", params);
            Ok(params)
        } else {
            error!("jmxfile参数文件不存在: {}", self.args.jmxfile.display());
            Err(anyhow!("JMX文件{}不存在", self.args.jmxfile.display()))
        }
    }

    fn all_params(&self) -> Result<Vec<String>> {
        let mut params = Vec::new();
        params.append(&mut self.overall_granularity_params());
        params.append(&mut self.propfile_params()?);
        params.append(&mut self.jmeterproperty_params());
        params.append(&mut self.thread_group_params());
        params.append(&mut self.jmxfile_params()?);

        debug!("all jmeter params: {:?}", params);
        Ok(params)
    }

    // 读取JMeter生成的HTML报告中简要的测试结果
    fn statistics_total_short(&self) -> Result<String> {
        let mut statistics_path = self.args.tmpdir.to_path_buf();
        statistics_path.push("res/statistics.json");
        let file = File::open(&statistics_path).map_err(|e|anyhow!("{}，{}", statistics_path.display(), e))?;
        // let file = File::open("0330-1752/res/statistics.json").map_err(|e|anyhow!("{}，{}", statistics_path.display(), e))?;
        let reader = BufReader::new(file);
        // Read the JSON contents of the file as an instance of `User`.
        let statistics: serde_json::Value = serde_json::from_reader(reader)?;
        let total_throughput = (&statistics["Total"]["throughput"].as_f64().unwrap()*10.0).trunc() as i64/ 10;
            // .as_f64().ok_or(|e| anyhow!("statistics.json转化toaol_throughput失败，{}", e))?
        let total_mean_res_time = (&statistics["Total"]["meanResTime"].as_f64().unwrap()*10.0).trunc() as i64/ 10;
        let total_error_count = &statistics["Total"]["errorCount"].as_i64().unwrap();
        debug!("总吞吐率: {}", total_throughput);
        debug!("平均响应时间: {}", total_mean_res_time);
        debug!("错误数: {}", total_error_count);

        Ok(format!("{}qps{}ms{}err", total_throughput, total_mean_res_time, total_error_count))
    }

    fn outputfolder(&self) -> Result<PathBuf> {
        let mut res = if let Some(outputfolder) = &self.args.outputfolder {
            outputfolder.display().to_string()
        }else {
            // egg: 0330-1752_1u_0qps0ms0err
            format!("{}_{}u_{}", self.args.tmpdir.display(), self.args.thread_num, self.statistics_total_short()?)
        };

        if let Some(ref append) = self.args.append {
            res = format!("{}_{}", res, append);
        }

        Ok(Path::new(res.as_str()).to_path_buf())
    }

    pub fn run(&self) -> Result<PathBuf> {
        // 存在则删除
        let tmpdir = self.args.tmpdir.as_path();
        if tmpdir.try_exists()? && tmpdir.is_dir() {
            warn!("临时归档目录存在，强制删除它: {}", tmpdir.display());
            std::fs::remove_dir_all(tmpdir)?;
        }

        // 运行JMeter
        let jmeter_cmd = if cfg!(target_os = "windows") {
            "jmeter.bat"
        } else {
            "jmeter"
        };

        // 调用JMeter
        let res = super::call_command(jmeter_cmd, self.all_params()?);

        let oldir = self.args.tmpdir.to_path_buf();
        let outputdir = match res {
            Ok(_) => { // 调用JMeter没有出错，正常结束
                // 重命名归档目录
                let newdir = self.outputfolder()?;
                if oldir.try_exists()? && oldir.is_dir() {
                    if newdir.try_exists()? && newdir.is_dir() {
                        error!("重命名归档目录: {} 时，新的归档已目录存在: {}, 结果仍在: {1} 中", oldir.display(), newdir.display());
                        // std::fs::rename(oldir, &newdir)?;
                        oldir
                    }else {
                        info!("重命名归档目录: {} -> {}", oldir.display(), newdir.display());
                        std::fs::rename(oldir, &newdir)?;
                        newdir
                    }
                }else {
                    error!("默认的归档目录不存在: {}", oldir.display());
                    Err(anyhow!("默认的归档目录不存在: {}", oldir.display()))?
                }
            },
            Err(e)=> { // 可能因JMeter HEAP 堆大小太小，导致JMeter HTML报告没生成，进程挂了
                error!("调用JMeter出错了：{}", e);
                oldir
            },
        };
        Ok(outputdir)
    }
// ## 计算远程服务器需监视的时长
// monitor_duration=`expr $Rampup + $Duration`
// monitor_duration=`expr $monitor_duration + 3`
// log info "动态计算，增加3秒jmeter启动时间后，monitor_duration: $monitor_duration "
// ## 计算远程服务器上运行nmon监视的间隔时间和监视次数
// monitor_interval=`expr $monitor_duration / 1440`
// monitor_interval=`expr $monitor_interval + 1`
// log info "动态计算，monitor_interval: $monitor_interval"
// monitor_count=`expr $monitor_duration / $monitor_interval`
// log info "动态计算，monitor_count: $monitor_count"
    pub fn calc_monitor_params(&self) -> (u64, u64) {
        let mut duration = self.args.rampup + self.args.duration;
        duration += 3;
        debug!("动态计算，增加3秒jmeter启动时间后，duration: {}", duration);
        let mut interval = duration / 1440;
        interval += 1;
        debug!("动态计算，interval: {}", interval);
        let count = duration / interval;
        (interval, count)
    }

}
