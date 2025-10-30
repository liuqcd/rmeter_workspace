pub mod server;

use crate::ssh::server::ClientInfo;
use anyhow::Result;
use anyhow::anyhow;
use async_ssh2_tokio::AuthMethod;
use async_ssh2_tokio::client::{Client, ServerCheckMethod};
use clap::{Parser, Subcommand, arg};
use log::debug;
use log::error;
use log::info;
use regex::Regex;
use server::Server;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tokio::task::JoinSet;
use tokio::time::timeout;
use tokio::process::Command;


pub const JMETER_DEFAULT_CONFIG_FILE: &str = "test.properties";
pub const SSH_DEFAULT_CONFIG_FILE: &str = "server.json";

#[derive(Parser, Debug, Clone)]
pub struct SshArgs {
    /// 正则表达式，代表使用ssh2批量操作的远程服务器，匹配配置文件中valid为true的信息，匹配规则:[$group.name || $group.name.hostname || $group.name.ip]， 默认值为".*
    #[arg(default_value = ".*")]
    #[arg(long = "nmon", value_name = "REGEX")]
    #[arg(long, value_name = "REGEX")]
    ssh_regex: Option<Regex>,
    /// ssh连接的配置文件
    #[arg(long, value_name = "FILE", default_value = "server.json")]
    #[arg(long, value_name = "FILE", default_value = SSH_DEFAULT_CONFIG_FILE)]
    ssh_config: Option<PathBuf>,
}

impl SshArgs {
    pub fn regex_mut(&mut self, regex: Regex) {
        self.ssh_regex = Some(regex);
    }
    pub fn regex(&self) -> Option<Regex> {
        match self.ssh_regex {
            None => Some(Regex::new(".*").unwrap()),
            Some(ref regex) => Some(regex.clone()),
        }
    }
    pub fn config(&self) -> PathBuf {
        self.ssh_config.clone().unwrap()
    }
}

#[derive(Subcommand, Debug, Clone)]
pub enum SshOps {
    /// 在远程服务器上执行命令
    Exec {
        /// 在远程服务器上执行的命令，如: ls perf/
        /// ls -- -l perf/
        statement: Vec<String>,
    },
    /// 从远程服务器上下载单个文件到本地目录
    Get {
        /// 要从远程服务器下载文件，只能为单个文件且不能为目录
        src_file_path: PathBuf,
        /// 下载文件保存在本地的目录，并重命名文件名格式为: [hostname]_[ip]_[filename], 建议使用相对路径。
        dest_dir: PathBuf,
    },
    /// 上传本地单个文件到远程服务器上
    Put {
        /// 要上传到远程服务器上的文件，只能为单个文件且不能为目录
        src_file_path: PathBuf,
        /// 上传到远程服务器上的目录, 要求为相对登录用户HOME目录的路
        dest_dir: PathBuf,
    },
    /// 打印server.json的模板json
    Print,
}

// 解析配置文件
pub fn parse_server_json(path: PathBuf) -> Result<Server> {
    let json = fs::read_to_string(path.clone())
        .map_err(|e| anyhow::Error::msg(format!("读取配置文件: {:?} 失败: {}", path, e)))?;
    let server: Server = serde_json::from_str(&json)
        .map_err(|e| anyhow::Error::msg(format!("反序列化配置文件: {:?} 失败: {}", path, e)))?;
    Ok(server)
}

pub async fn run(ops: SshOps, client_info: ClientInfo) -> Result<()> {
    let mut set = JoinSet::new();
    client_info.clone_info().into_iter().for_each(|i| {
        let ops = ops.clone();
        set.spawn(async {
            match ops {
                SshOps::Exec { statement } => exec(i, statement.clone()).await,
                SshOps::Get { src_file_path, dest_dir, } => get(i, src_file_path.clone(), dest_dir.clone()).await,
                SshOps::Put { src_file_path, dest_dir, } => put(i, src_file_path.clone(), dest_dir.clone()).await,
                SshOps::Print => {
                    let res = "此处已经在main函数最开始处理，请勿重复调用！";
                    error!("{}", res);
                    Ok(res.to_string())
                }
            }
        });
    });

    while let Some(res) = set.join_next().await {
        match res? {
            Ok(s) => info!("{}", s),
            Err(e) => error!("{}", e),
        }
    }

    Ok(())
}


async fn exec(info: crate::ssh::server::Info, statement: Vec<String>) -> Result<String> {
    let res = match info.auth_method() {
        Some(auth_method) => {
            let ops_impl = Ssh2Ops::new(auth_method);
            ops_impl.exec(info, statement).await?
        },
        None => {
            let ops_impl = LocalSsh::new();
            ops_impl.exec(info, statement).await?
        },
    };
    Ok(res)
}
async fn put(info: crate::ssh::server::Info, src_file_path: PathBuf, dest_dir: PathBuf,) -> Result<String> {
    let res = match info.auth_method() {
        Some(auth_method) => {
            let ops_impl = Ssh2Ops::new(auth_method);
            ops_impl.upload_file(info, src_file_path, dest_dir).await?
        },
        None => {
            let ops_impl = LocalSsh::new();
            ops_impl.upload_file(info, src_file_path, dest_dir).await?
        },
    };
    Ok(res)
}
async fn get(info: crate::ssh::server::Info, src_file_path: PathBuf, dest_dir: PathBuf,) -> Result<String> {
    let res = match info.auth_method() {
        Some(auth_method) => {
            let ops_impl = Ssh2Ops::new(auth_method);
            ops_impl.download_file(info, src_file_path, dest_dir).await?
        },
        None => {
            let ops_impl = LocalSsh::new();
            ops_impl.download_file(info, src_file_path, dest_dir).await?
        },
    };
    Ok(res)
}


trait RemoteOps {
    async fn exec(&self, info: crate::ssh::server::Info, statement: Vec<String>) -> Result<String>;
    async fn upload_file(&self, info: crate::ssh::server::Info, src_file_path: PathBuf, dest_dir: PathBuf,) -> Result<String>;
    async fn download_file(&self, info: crate::ssh::server::Info, src_file_path: PathBuf, dest_dir: PathBuf,) -> Result<String>;
}


pub struct Ssh2Ops {
    auth_method: AuthMethod,
}

impl Ssh2Ops {
    pub fn new(auth_method: AuthMethod) -> Self {
        Ssh2Ops {
            auth_method,
        }
    }
    async fn connect(&self, ip: &str, port: u16, username: &str) -> Result<Client> {
        debug!("auth_method:{:?}", self.auth_method);
        let client = timeout(
            Duration::from_secs(10),
            Client::connect(
                (ip, port),
                username,
                self.auth_method.clone(),
                ServerCheckMethod::NoCheck,
                // ServerCheckMethod::DefaultKnownHostsFile,
                // ServerCheckMethod::KnownHostsFile("~/.ssh/known_hosts".to_string()),
            ),
        )
        .await
        .map_err(|e| anyhow!("连接远程服务器超时: {}, 失败: {}", ip, e))?
        .map_err(|e| anyhow!("连接远程服务器: {}, 失败: {}", ip, e))?;
        Ok(client)
    }
}

impl RemoteOps for Ssh2Ops {

    async fn exec(&self, info: crate::ssh::server::Info, statement: Vec<String>) -> Result<String> {
        let client = self.connect(info.ip(), info.port(), info.username()).await?;
        let mut command = String::new();
        statement
            .iter()
            .for_each(|a| command.push_str(&format!(" {}", a)));
        debug!("command: {}", command);
        let result = client
            .execute(command.as_str())
            .await
            .map_err(|e| anyhow!("执行远程命令: {}, 失败: {}", info.ip(), e))?;
        let output = if result.stdout.is_empty() {
            result.stderr
        } else {
            result.stdout
        };
        // format!("{}, 远程命令: {}, 执行结果:\n{}", info, command, output.trim())
        let res = format!("{}, 远程命令:{}，执行返回状态:{}，执行结果:\n{}", info, command, result.exit_status, output );
        Ok(res)
    }

    async fn upload_file(&self, info: crate::ssh::server::Info, src_file_path: PathBuf, dest_dir: PathBuf,) -> Result<String> {
        let client = self.connect(info.ip(), info.port(), info.username()).await?;
        let dest_file_path = match dest_dir.extension() {
            None =>  {
                dest_dir.join(src_file_path.file_name()
                                                .ok_or(anyhow!("{} 获取文件名失败: {}", info, src_file_path.display()))?
                                        ).display().to_string().replace('\\', "/")
                                            },
            Some(_) => dest_dir.display().to_string().replace('\\', "")
        };

        client.upload_file(&src_file_path, &dest_file_path).await?;
        let res = format!( "{}, upload {} to {} success", info, src_file_path.display(), dest_file_path );
        Ok(res)
    }

    async fn download_file(&self, info: crate::ssh::server::Info, src_file_path: PathBuf, dest_dir: PathBuf,) -> Result<String> {
        let client = self.connect(info.ip(), info.port(), info.username()).await?;
        let path = if dest_dir.is_dir() {
            src_file_path.clone()
        } else {
            dest_dir.clone()
        };
        let dest_file_name = path.file_name().ok_or(anyhow!("{}, 获取文件名失败: {}", info, path.display()))?;
        let dest_file_path = dest_dir.join(format!(
            "{}_{}",
            info.hostname_ip(),
            dest_file_name.display()
        ));
        client.download_file(&src_file_path.display().to_string().replace('\\', "/"), &dest_file_path).await?;
        let res = format!( "{}, download {} to {} success", info, src_file_path.display(), dest_file_path.display());
        Ok(res)
    }
}


pub struct LocalSsh {}

impl LocalSsh {
    pub fn new() -> Self {
        LocalSsh {}
    }
}
impl RemoteOps for LocalSsh{

    async fn exec(&self, info: crate::ssh::server::Info, statement: Vec<String>) -> Result<String> {
        let mut command= Command::new("ssh");
        command.arg(format!("{}@{}", info.username(), info.ip()));
        command.args(statement);

        let output = command.output().await?;
        let res_vec = if output.status.success() {
            output.stdout
        } else {
            output.stderr
        };
        let res = format!("{}, 远程命令:{:?}，执行返回状态:{}，执行结果:\n{}", info, command, output.status, String::from_utf8_lossy(res_vec.as_ref()) );
        Ok(res)
    }

    async fn upload_file(&self, info: crate::ssh::server::Info, src_file_path: PathBuf, dest_dir: PathBuf,) -> Result<String> {
        let mut dest_file_path = match dest_dir.extension() {
            None =>  {
                dest_dir.join(src_file_path.file_name()
                                                .ok_or(anyhow!("{} 获取文件名失败: {}", info, src_file_path.display()))?
                                        ).display().to_string().replace('\\', "/")
                                            },
            Some(_) => dest_dir.display().to_string().replace('\\', "")
        };

        dest_file_path = if dest_file_path.starts_with("/") || dest_file_path.starts_with("~") {
            dest_file_path
        }else {
            format!("~/{}", dest_file_path)
        };

        let mut command= Command::new("scp");
        command.arg("-r");
        command.arg(&src_file_path);
        command.arg(format!("{}@{}:{}", info.username(), info.ip(), dest_file_path));

        let output = command.output().await?;
        let res = format!( "{}, upload {} to {}, status: {}", info, src_file_path.display().to_string().replace('\\', ""), dest_file_path, output.status );
        Ok(res)
    }

    async fn download_file(&self, info: crate::ssh::server::Info, src_file_path: PathBuf, dest_dir: PathBuf,) -> Result<String> {
        let src_file_path_string = if src_file_path.starts_with("/") || src_file_path.starts_with("~") {
            src_file_path.display().to_string().replace('\\', "")
        }else {
            format!("~/{}", src_file_path.display().to_string().replace('\\', ""))
        };

        let path = if dest_dir.is_dir() {
            src_file_path.clone()
        } else {
            dest_dir.clone()
        };

        let dest_file_name = path.file_name().ok_or(anyhow!("{}, 获取文件名失败: {}", info, path.display()))?;

        let dest_file_path = dest_dir.join(format!(
            "{}_{}",
            info.hostname_ip(),
            dest_file_name.display()
        ));

        let dest_file_path_string = dest_file_path.display().to_string().replace('\\', "");

        let mut command= Command::new("scp");
        command.arg("-r");
        command.arg(format!("{}@{}:{}", info.username(), info.ip(), src_file_path_string));
        command.arg(&dest_file_path_string);

        let output = command.output().await?;
        let res = format!( "{}, download {} to {}, status: {}", info, src_file_path_string, dest_file_path_string, output.status);
        Ok(res)
    }
}
