use clap::{ArgAction, Parser, arg};
use chrono::{offset::Local};
use log::info;
use log::debug;
use std::time::SystemTime;
use anyhow::Ok;
use anyhow::Result;
use anyhow::anyhow;
use std::path::PathBuf;
use std::path::Path;

/// 使用rscript按一定规范创建开发JMeter脚本工作目录，并传入一些有利于脚本开发的参数后打开JMeter
///
/// 要求：
///   1. rscript和jmeter均配在Path环境变量中。
///   2. 可配置环境变量: RMETER_TEMPLATE_DIR
///   3. 工具堆: JMeter + nmon + rnmon
#[derive(Parser, Debug)]
#[command(author = "liuqxx", version = "0.1.0")]
pub struct Args {
    // /// 指定一个日志输出文件(追加)，默认nmon、ssh子命令只输出dubug级别控制台的日志，jmeter子命令输出info日志到控制台和日志文件
    // #[arg(short, long, value_name = "FILE", default_value = "run.log")]
    // pub logfile: Option<PathBuf>,

    /// 一个开启DEBUG日志，两个及以上开启trace日志
    #[arg(short, long, action = ArgAction::Count)]
    pub debug: u8,

    #[command(flatten)]
    pub script_args: ScriptArgs,
}

#[derive(Parser, Debug, Clone)]
pub struct ScriptArgs {
    /// 开启外置模板文件夹，该文件中包含ssh,nmon,jmx和三方包的模板文件
    #[arg(short, long, value_name = "DIR", env = "RMETER_TEMPLATE_DIR",)]
    pub template_dir: Option<PathBuf>,
    /// 若目录存在，则删除该目录后，重新创建。
    #[arg(short, long, action = ArgAction::SetTrue)]
    force: Option<bool>,
    /// 是否复制test.properties文件到工作目录下scrip目录
    #[arg(short, action = ArgAction::SetTrue)]
    properties: Option<bool>,
    /// 新建工作目录，并在目录下按规范生成一些文件。
    name: String,
}

fn main() -> Result<()> {
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

    log = match cli_args.debug {
        0 => log.level(log::LevelFilter::Info),
        1 => log.level(log::LevelFilter::Debug),
        _ => log.level(log::LevelFilter::Trace),
    };

    info!("cli_args: {:?}", &cli_args);
    log.apply()?;

    // 业务参数
    let args = cli_args.script_args;

    // 目标目录
    let work_dir = Path::new(&args.name);
    let script_dir = work_dir.join("script");
    let reference_dir = work_dir.join("参考资料");
    let script_data_dir = script_dir.join("data");

    // template_dir
    match args.template_dir {
        None => {
            return Err(anyhow!("命令行参数\"-t, --template-dir\"参数未设置 且 无环境变量: RMETER_TEMPLATE_DIR"));
        },
        Some(ref dir) => {
            if !dir.exists() || !dir.is_dir() {
                return Err(anyhow!(
                    "命令行参数\"-t, --template-dir\"设置的\"{}\" not exists or not is dir.",
                    dir.display()
                ));
            }
        }
    }

    let tempdir = args.template_dir.clone().unwrap();
    let tempdir_jmeter = tempdir.join("jmeter");
    debug!("template_dir_jmeter: {:?}", tempdir_jmeter);
    let tempdir_nmon = tempdir.join("nmon");
    debug!("template_dir_nmon: {:?}", &tempdir_nmon);
    let tempdir_ssh = tempdir.join("ssh");
    debug!("template_dir_ssh: {:?}", &tempdir_ssh);

    // force
    if let Some(ref force) = args.force {
        if *force && work_dir.exists() && work_dir.is_dir() {
            std::fs::remove_dir_all(work_dir)?;
        }
    }
    // 创建目录
    // 创建性能测试工作目录
    std::fs::create_dir(&work_dir).map_err(|e| anyhow!("work_dir: {:?}, {}", &work_dir, e))?;
    // 创建性能测试脚本目录
    std::fs::create_dir(&script_dir)?;
    // 创建性能测试脚本数据目录
    std::fs::create_dir(&script_data_dir)?;
    // 创建性能测试参考目录
    std::fs::create_dir(&reference_dir)?;
    
    // copy文件
    // copy jmx模板文件
    std::fs::copy(
        tempdir_jmeter.join("test.jmx"),
        script_dir.join("test.jmx"),
    )?;
    // copy 测试配置模板文件
    if let Some(ref properties) = args.properties {
        if *properties {
            std::fs::copy(
                tempdir_jmeter.join("test.properties"),
                script_dir.join("test.properties"),
            )?;
        }
    }
    // copy 监控模板文件
    std::fs::copy(
        tempdir_ssh.join("server.json"),
        script_dir.join("server.json"),
    )?;
    // copy readme.txt
    std::fs::copy(
        tempdir_jmeter.join("readme.txt"),
        work_dir.join("readme.txt"),
    )?;

    info!("使用命令切换在脚本目录： cd \"{}\"", &script_dir.display());

    Ok(())
}
