mod client;
mod jmeter;

use anyhow::Result;
use anyhow::anyhow;
use chrono::Local;
use clap::Parser;
use log::debug;
use log::info;
use log::error;
use std::path::PathBuf;
use std::process::Child;
use std::process::Command;
use std::time::SystemTime;

use crate::client::Args;
use crate::jmeter::JMeter;

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

    log.apply()?;
    let jmeter_args = cli_args.jmeter_args.clone();
    let ssh_args = cli_args.ssh_args.clone();
    let nmon_args = cli_args.nmon_args.clone();
    debug!("cli_args: {:?}", cli_args);
    debug!("ssh_args: {:?}", ssh_args);
    debug!("nmon_args: {:?}", nmon_args);

    let jmeter = JMeter::new(jmeter_args.clone());

    // let mut server_nmon_dir = PathBuf::new();
    let mut server_nmon_file = String::new();

    let child_rssh = if ssh_args.nmon {
        let ssh_dir = ssh_args.ssh_dir.clone().expect("调用rssh程序时其ssh_dir参数无值");

        info!("各服务上，查询可能与我们约定有冲突的nmon监控");
        call_command( "rssh", vec![ "exec".to_string(), "--".to_string(), format!("ps -ef|grep {} | grep nmon |grep -v grep", ssh_dir), ], )?;

        let kill_statement = format!(
            r#"ps -ef | grep {} | grep nmon | grep -v jmx | grep -v grep | awk '{{print $2}}' | xargs -r kill -9"#,
            ssh_dir
        );
        info!( "各服务上，杀掉与我们约定有冲突的nmon监控: {}", &kill_statement );
        call_command( "rssh", vec![ "exec".to_string(), "--".to_string(), format!("ps -ef|grep {} | grep nmon |grep -v grep", ssh_dir), ], )?;

        let tmp_dir = jmeter.get_tmp_dir();

        let mut server_nmon_dir = PathBuf::from(&ssh_dir);
        server_nmon_dir.push(&tmp_dir);
        // #[cfg(target_os = "windows")]
        let mkdir_statement = format!(r#"mkdir -p {}"#, server_nmon_dir.display().to_string().replace('\\', "/"));
        // #[cfg(not(target_os = "windows"))]
        // let mkdir_statement = format!(r#"mkdir -p {}"#, server_nmon_dir.display().to_string());
        info!( "各服务上，新建nmon监控的工具目录，mkdir: {}", &mkdir_statement );
        call_command( "rssh", vec!["exec".to_string(), "--".to_string(), mkdir_statement], )?;

        // let no_wait = sshargs.
        server_nmon_dir.push("res.nmon");
        server_nmon_file = server_nmon_dir.display().to_string().replace('\\', "/");
        let (interval, count) = jmeter.calc_monitor_params();
        let run_statement = format!( r#"nmon -F {} -t -s {} -c {}"#, server_nmon_file, interval, count );
        info!("各服务上，运行我们的nmon监控: {}", &run_statement);

        if ssh_args.nowait {
            let child_rssh = call_command_nowait( "rssh", vec!["exec".to_string(), "--".to_string(), run_statement], )?;
            Some(child_rssh)
        }else {
            call_command( "rssh", vec!["exec".to_string(), "--".to_string(), run_statement], )?;
            None
        }
    }else {
        None
    };

    // 运行JMeter
    info!("jmeter_args: {:?}", jmeter_args);
    // todo 根据运行时间动态设置JMeter的HEAP的环境变量
    let jmeter_output_dir = jmeter.run()?;

    //处理可能与JMeter并行运行的rssh子程序
    if let Some(mut child) = child_rssh {
        match child.try_wait() {
            Ok(Some(status)) => debug!("exited with: {status}"),
            Ok(None) => {
                debug!("status not ready yet, let's kill it");
                child.kill().expect("command couldn't be killed");
            }
            Err(e) => error!("error attempting to wait rssh command: {e}"),
        }
    }

    if ssh_args.nmon {
        info!("下载nmon文件...");
        let local_nmon_dir = jmeter_output_dir.join("nmon").display().to_string().replace('\\', "/");
        std::fs::create_dir_all(&local_nmon_dir)?;
        call_command( "rssh", vec![ "get".to_string(), server_nmon_file.clone(), local_nmon_dir.clone(), ], )?;

        if let Some(nmon_args) = nmon_args.clone() {
            info!("分析nmon文件...");
            let mut params = nmon_args.params();
            // 增加分析后的结果目录
            params.push("--html-output".to_string());
            params.push(jmeter_output_dir.display().to_string());
            // 增加分析的nmon目录
            params.push(local_nmon_dir);
            call_command("rnmon", params)?;
        }

        info!("删除远程服务器上的nmon文件...");
        call_command( "rssh", vec![ "exec".to_string(), "--".to_string(), "rm -r".to_string(), server_nmon_file, ], )?;
    }

    let logfile = cli_args.logfile.clone().unwrap();
    std::fs::copy(&logfile, jmeter_output_dir.join("run.log"))?;
    std::fs::remove_file(logfile)?;

    info!( "运行结束，结果数据在此目录下: {}", jmeter_output_dir.display() );

    Ok(())
}

fn call_command(name: &str, args: Vec<String>) -> Result<()> {
    let mut command = Command::new(name);
    command.args(args);
    info!("调用子命令程序: {}, 其参数:{:?}", name, &command);
    command
        .spawn()
        .map_err(|e| anyhow!("调用子命令程序失败: {}， 子程序: {:?} ", e, command))?
        .wait()?;
    Ok(())
}
fn call_command_nowait(name: &str, args: Vec<String>) -> Result<Child> {
    let mut command = Command::new(name);
    command.args(args);
    info!("调用子命令程序: {}, 其参数:{:?}", name, &command);
    let child = command
        .spawn()
        .map_err(|e| anyhow!("调用子命令程序失败: {}， 子程序: {:?} ", e, command))?
        ;
    Ok(child)
}
