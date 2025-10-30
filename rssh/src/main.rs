pub mod ssh;

use anyhow::Ok;
use anyhow::Result;
use anyhow::anyhow;
use chrono::offset::Local;
use clap::{ArgAction, Parser, arg};
use log::debug;
use log::info;
use std::path::PathBuf;
use std::time::SystemTime;
use crate::ssh::SshArgs;
use crate::ssh::SshOps;

/// 利用tokio异步机制，可在一堆远程服务器上执行linux命令，以及上传和下载单个文件。
/// 所有连接顺序连接成功后同时执行命令
#[derive(Parser, Debug)]
#[command(author = "liuqxx", version = "0.1.0")]
pub struct Args {
    /// 指定一个日志输出文件(追加)，默认nmon、ssh子命令只输出dubug级别控制台的日志，jmeter子命令输出info日志到控制台和日志文件
    #[arg(short, long, value_name = "FILE", default_value = "run.log")]
    pub logfile: Option<PathBuf>,
    /// 一个开启DEBUG日志，两个及以上开启trace日志
    #[arg(short, long, action = ArgAction::Count)]
    pub debug: u8,

    #[command(flatten)]
    pub ssh_args: SshArgs,
    /// 2
    #[command(subcommand)]
    pub ops: SshOps,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 处理传入的程序的参数
    let cli_args = Args::parse();


    if let SshOps::Print = &cli_args.ops {
        // println!("{}", crate::ssh::server::ser());
        let prety_str = r#"
{"groups": [
  {"name": "模板", "members": [
    {"hostname": "redis", "ip": "192.168.1.2", "port": 22, "user": "cx", "auth": {"Password": "chaxun"}, "valid": true},
    {"hostname": "redis", "ip": "192.168.1.2", "port": 22, "user": "cx", "auth": "LocalSsh", "valid": true},
    {"hostname": "redis", "ip": "192.168.1.2", "port": 22, "user": "cx", "auth": {"PrivateKeyFile": {}},"valid": true},
    {"hostname": "redis", "ip": "192.168.1.2", "port": 22, "user": "cx", "auth": {"PrivateKeyFile": {"key_file_path": "~/.ssh/ed25519"}}, "valid": true},
    {"hostname": "redis", "ip": "192.168.1.2", "port": 22, "user": "cx", "auth": {"PrivateKeyFile": {"key_file_path": "~/.ssh/ed25519","key_pass": "pass"}}, "valid": true}
  ], "valid": false}
]}
"#;
        println!("{}", prety_str);
        return Ok(());
    };

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

    log = if let Some(ref logfile) = cli_args.logfile {
        log.chain(fern::log_file(logfile)?)
    } else {
        log
    };

    log = match cli_args.debug {
        0 => log.level(log::LevelFilter::Info),
        1 => log.level(log::LevelFilter::Debug),
        _ => log.level(log::LevelFilter::Trace),
    };

    info!("cli_args: {:?}", &cli_args);
    log.apply()?;

    // 业务参数
    let ssh_args = cli_args.ssh_args;
    let ops = cli_args.ops;
    info!("ssh_args: {:?}", ssh_args);
    info!("ops: {:?}", ops);


    let config = ssh_args.config();
    // let auth = ssh_args.auth();
    // 读取配置文件
    if let Some(ref regex) = ssh_args.regex() {
        debug!("regex: {:?}", regex);
        let server = ssh::parse_server_json(PathBuf::from(config))?;
        debug!("server: {:?}", server);
        let client_info = server
            .client_info(regex)
            .ok_or(anyhow!("配置文件中，没找到匹配: {} 的信息", regex))?;
        debug!("Ops: {:?}, client_info: {:?}", ops, client_info);

        crate::ssh::run(ops, client_info).await?
    }

    Ok(())
}
