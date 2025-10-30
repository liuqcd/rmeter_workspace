use chrono::Local;
use clap::{ArgAction, Parser, arg};
use std::path::PathBuf;

const JMETER_DEFAULT_CONFIG_FILE: &str = "test.properties";


/// 使用rmeter，减少使用JMeter CLI MOD压测时的一些重复性操作，让我们更关注结果本身
///
/// 简单的测试流程如下：
///   1. 删除/归档上次压测时产生的文件，包括JMeter和nmon监控数据，分布在压力机上和被监控服务器上。
///   2. 发起JMeter压测，发起nmon监控。
///   3. 下载nmon的监控文件，分析监控文件。
///   4. 归档测试数据，包括JMeter产生的数据，nmon监控数据。
#[derive(Parser, Debug)]
#[command(author = "liuqxx", version = "0.1.0")]
pub struct Args {
    /// 指定一个日志输出文件(追加)，默认nmon、ssh子命令只输出dubug级别控制台的日志，jmeter子命令输出info日志到控制台和日志文件
    #[arg(long, value_name = "FILE", default_value = "run.log")]
    pub logfile: Option<PathBuf>,

    /// 一个开启DEBUG日志，两个及以上开启trace日志
    #[arg(long, action = ArgAction::Count)]
    pub debug: u8,

    /// the jmeter test(.jmx) file to run. "-t LAST" will load last。
    /// 与jmeter -n -t test.jmx 等价
    #[command(flatten)]
    pub jmeter_args: JMeterArgs,

    /// 到各服务器上生成nmon监控数据，REGEX匹配server.json文件中groupname、hostname和ip三项任一匹配即可
    #[command(flatten)]
    pub ssh_args: SshArgs,

    #[command(flatten)]
    pub nmon_args: Option<NmonArgs>,
}

#[derive(Parser, Debug, Clone)]
#[command(next_help_heading = "Ssh")]
/// 子程序调用rssh程序
pub struct SshArgs {
    /// 设置nmon监控
    #[arg(long, action = ArgAction::SetTrue)]
    pub nmon: bool,
    /// nmon监控时，设置是否立即执行JMeter
    #[arg(long, action = ArgAction::SetTrue)]
    pub nowait: bool,
    /// 服务器上nmon结果存放的目录，也是识别nmon进程的标识，尽量唯一，只有当-nmon 生效的时候才需要
    #[arg(long = "nmondir", value_name = "DIR", default_value = "perf")]
    pub ssh_dir: Option<String>,
}

#[derive(Parser, Debug, Clone)]
#[command(next_help_heading = "Nmon")]
pub struct NmonArgs {
    /// 对输入的nmon文件进行分析，并生成HTML图表，当指定html时cpu默认为true
    #[arg(long, action = ArgAction::SetTrue)]
    pub html: bool,
    /// 分析nmon文件时，设置对cpu使用进行分析
    #[arg(long, default_value_t = true)]
    pub cpu: bool,
    /// 分析nmon文件时，设置对disk_busy使用进行分析
    #[arg(long, action = ArgAction::SetTrue)]
    pub disk_busy: bool,
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
    /// 分析nmon文件时，设置对disk_io使用进行分析，暂不支持todo
    #[arg(long, action = ArgAction::SetTrue)]
    pub disk_io: bool,
    /// 分析nmon文件时，设置对disk_summary使用进行分析，暂不支持todo
    #[arg(long, action = ArgAction::SetTrue)]
    pub disk_summary: bool,
    /// 分析nmon文件时，设置对net使用进行分析，不支持todo
    #[arg(long, action = ArgAction::SetTrue)]
    pub net: bool,
}
impl NmonArgs {
    pub fn params(&self) -> Vec<String> {
        let mut params = Vec::new();

        if self.html {
            params.push("--html".to_string());
        }
        if self.cpu {
            params.push("--cpu".to_string());
        }
        if self.disk_busy {
            params.push("--disk-busy".to_string());
        }
        if self.mem_free {
            params.push("--mem-free".to_string());
        }
        if self.mem_active {
            params.push("--mem-active".to_string());
        }
        if self.mem_swap {
            params.push("--mem-swap".to_string());
        }
        if self.jfsfile {
            params.push("--jfsfile".to_string());
        }
        if self.disk_io {
            params.push("--disk-io".to_string());
        }
        if self.disk_summary {
            params.push("--disk-summary".to_string());
        }

        params
    }
}

#[derive(Parser, Debug, Clone)]
#[command(next_help_heading = "JMeter")]
pub struct JMeterArgs {
    #[arg(short, long, value_name = "FILE")]
    pub jmxfile: PathBuf,
    /// 与jmeter -JThreads=<num> 等价
    #[arg(short, long, default_value = "1")]
    pub thread_num: usize,
    /// 线程加载时间，单位：秒。
    // /// 与jmeter -JRampup=<sesc> 等价
    // #[arg(short, long, default_value = "1")]
    // rampup: i32,
    /// 与jmeter -JRampup=<sesc> 等价
    #[arg(short, long, default_value = "1")]
    pub rampup: u64,
    /// 1. Thread Group线程组时，代表: Loop Count , 有效值>=-1。
    /// 2. Concurrency Thread Group线程组时，代表： Ramp-Up Steps Count , 有效值>=1。
    /// 与jmeter -JLoopOrRampupCount=<count> 等价
    #[arg(short, long, default_value = "-1")]
    pub count: i32,
    /// 线程加载完后的运行时间。
    /// 与jmeter -JDuration=<sesc> 等价
    #[arg(short, long, default_value = "1")]
    pub duration: u64,
    /// 测试配置文件，常用用但不咋个改的配置。若不指定，则使用rmeter内置的配置信息。
    #[arg(short, long, value_name="FILE", default_value = JMETER_DEFAULT_CONFIG_FILE)]
    /// 与jmeter -Jkey1=value1 -Jkey2=value2 等价, key和value均为配置文件里的值
    pub propfile: Option<PathBuf>,
    /// JMeter -J<argument>=<value> Define additional JMeter properties。
    /// 与jmeter -Jkey1=value1 等价，命令行传入的参数，会覆盖配置文件里的参数。
    #[arg(short = 'J', long, action = ArgAction::Append)]
    pub jmeterproperty: Option<Vec<String>>,

    // /// 后附一个或多个JMeter其他参数, 比如： -Rserver1,server2,…
    // #[arg(short, long, value_name = "string", action = ArgAction::Append)]
    // escape: Option<Vec<String>>,
    // todo
    /// 重命名输出目录，输入信息包含：JMeter的产物（JTL文件，jmeter.log，jmeter运行结束后的HTML报告目录），NMON监控产物（nmon文件，nmon生成的HTML文件）和一些三方依赖包。
    /// 默认，按月日时分的时间戳_JMeter部分输出信息（比如：0313-2212_1qps2ms0err）
    #[arg(short, long)]
    pub outputfolder: Option<PathBuf>,
    /// 输出目录后附加一段备注说明，比如输出目录为: 0313-2212_1qps2ms0err_test1，则append为test1
    #[arg(long)]
    pub append: Option<String>,
    // 临时输出目录，压力机输入临时目录和服务器上的时间输出目录
    #[arg(skip=format!("{}", Local::now().format("%m%d-%H%M")))]
    pub tmpdir: PathBuf,
}
