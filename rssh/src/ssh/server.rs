use async_ssh2_tokio::client::AuthMethod;
use regex::Regex;
use serde_derive::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

/// 代表整个json配置文件
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Server {
    // #[serde(flatten)]
    groups: Vec<Group>,
}

impl Server {
    /// 根据正则表达式查找匹配的配置文件
    pub fn client_info(&self, regex: &Regex) -> Option<ClientInfo> {
        if let Some(server) = self.find(regex) {
            Some(ClientInfo::new(server.groups))
        } else {
            None
        }
    }
    fn find(&self, regex: &Regex) -> Option<Self> {
        let mut groups = Vec::new();
        self.groups.iter().for_each(|g| {
            if let Some(s) = g.find(regex) {
                groups.push(s);
            }
        });
        if groups.is_empty() {
            None
        } else {
            Some(Self { groups })
        }
    }
}

impl Default for Server {
    fn default() -> Self {
        Self {
            groups: vec![Default::default()],
        }
    }
}

/// 多行连接信息可组成一组
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Group {
    name: String,
    members: Vec<Member>,
    valid: bool,
}

impl Group {
    fn find(&self, regex: &Regex) -> Option<Self> {
        if self.valid {
            let mut vec = Vec::new();
            if regex.is_match(&self.name) {
                self.members
                    .iter()
                    .filter(|a| a.valid())
                    .for_each(|a| vec.push(a.clone()));
            } else {
                self.members.iter().for_each(|a| {
                    if let Some(s) = a.find(regex) {
                        vec.push(s);
                    }
                });
            }
            if vec.is_empty() {
                None
            } else {
                Some(Self {
                    name: self.name.clone(),
                    members: vec,
                    valid: true,
                })
            }
        } else {
            None
        }
    }
}

impl Default for Group {
    fn default() -> Self {
        Self {
            name: Default::default(),
            members: vec![Default::default()],
            valid: true,
        }
    }
}

/// 每一行连接信息即一个Info实例。
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct Member {
    hostname: String,
    ip: String,
    port: u16,
    user: String,
    auth: Auth,
    valid: bool,
}

impl Member {
    fn valid(&self) -> bool {
        self.valid
    }
    fn find(&self, regex: &Regex) -> Option<Self> {
        if self.valid && (regex.is_match(&self.hostname) || regex.is_match(&self.ip)) {
            Some(self.clone())
        } else {
            None
        }
    }
}

impl Default for Member {
    fn default() -> Self {
        Self {
            hostname: Default::default(),
            ip: Default::default(),
            port: 22,
            user: String::from("cx"),
            auth: Auth::Password("chaxun".to_string()),
            valid: true,
        }
    }
}

impl fmt::Display for Member {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "hostname: {}, ip: {}, port: {}, user: {}, auth:{:?}, valid:{}",
            self.hostname, self.ip, self.port, self.user, self.auth, self.valid
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub enum Auth {
    Password(String),
    PrivateKey {
        key_data: String,
        key_pass: Option<String>,
    },
    PrivateKeyFile {
        key_file_path: Option<PathBuf>,
        key_pass: Option<String>,
    },
    LocalSsh,
}

#[derive(Debug, Clone)]
pub struct ClientInfo {
    info: Vec<Info>,
}

impl ClientInfo {
    fn new(groups: Vec<Group>) -> Self {
        let mut client_info = Self { info: Vec::new() };
        groups.into_iter().for_each(|g| client_info.append(g));
        client_info
    }

    fn append(&mut self, group: Group) {
        group.members.into_iter().for_each(|m| {
            self.info.push(Info {
                hostname: m.hostname,
                ip: m.ip,
                port: m.port,
                username: m.user,
                auth: m.auth.clone(),
                groupname: group.name.clone(),
            })
        });
    }
    pub fn clone_info(&self) -> Vec<Info> {
        self.info.clone()
    }
}

#[derive(Debug, Clone)]
pub struct Info {
    hostname: String,
    ip: String,
    port: u16,
    username: String,
    auth: Auth,
    groupname: String,
}

impl Info {
    pub fn hostname_ip(&self) -> String {
        format!("{}_{}", self.hostname, self.ip)
    }
    // pub fn hostname(&self) -> String {
    pub fn hostname(&self) -> &str {
        // self.hostname.to_string()
        self.hostname.as_str()
    }
    // pub fn ip(&self) -> String {
    pub fn ip(&self) -> &str {
        // self.ip.to_string()
        self.ip.as_str()
    }
    pub fn port(&self) -> u16 {
        self.port
    }
    // pub fn username(&self) -> String {
    pub fn username(&self) -> &str {
        // self.username.to_string()
        self.username.as_str()
    }
    // pub fn auth_method(&self) -> String {
    pub fn auth_method(&self) -> Option<AuthMethod> {
        let home = std::env::var("HOME") // Unix / macOS
            .or_else(|_| std::env::var("USERPROFILE"))
            .expect("Unix/macOS no $HOME or Windows no %USERPROFILE%"); // Windows

        let auth_method = match &self.auth {
            Auth::Password(pwd) => {
                let auth_method = AuthMethod::Password(pwd.to_string());
                Some(auth_method)
            },
            Auth::PrivateKey { key_data, key_pass } => {
                let auth_method = AuthMethod::PrivateKey {
                key_data: key_data.to_string(),
                key_pass: key_pass.clone(),
                };
                Some(auth_method)
            },
            Auth::PrivateKeyFile {
                key_file_path,
                key_pass,
            } => {
                // ~/.ssh/id_ed25519
                let private_key_file_path: PathBuf =
                    [home.clone(), ".ssh".into(), "id_ed25519".into()]
                        .iter()
                        .collect();
                let key_file_path = match key_file_path {
                    None => private_key_file_path,
                    Some(path) => path.to_path_buf(),
                };
                let auth_method = AuthMethod::PrivateKeyFile {
                    key_file_path,
                    key_pass: key_pass.clone(),
                };
                Some(auth_method)
            },
            Auth::LocalSsh  => None
        };
        auth_method
    }
    // pub fn groupname(&self) -> String {
    pub fn groupname(&self) -> &str {
        // self.groupname.to_string()
        self.groupname.as_ref()
    }
}

impl fmt::Display for Info {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.hostname_ip(),)
    }
}

pub fn ser() -> String {
    let m1 = Member {
        hostname: "redis".to_string(),
        ip: "192.168.1.2".to_string(),
        port: 22,
        user: "cx".to_string(),
        auth: Auth::Password("chaxun".to_string()),
        valid: true,
    };
    let m2 = Member {
        hostname: "redis".to_string(),
        ip: "192.168.1.2".to_string(),
        port: 22,
        user: "cx".to_string(),
        auth: Auth::LocalSsh,
        valid: true,
    };
    let m3 = Member {
        hostname: "redis".to_string(),
        ip: "192.168.1.2".to_string(),
        port: 22,
        user: "cx".to_string(),
        auth: Auth::PrivateKeyFile {
            key_file_path: None,
            key_pass: None,
        },
        valid: true,
    };
    let m4 = Member {
        hostname: "redis".to_string(),
        ip: "192.168.1.2".to_string(),
        port: 22,
        user: "cx".to_string(),
        auth: Auth::PrivateKeyFile {
            key_file_path: Some(PathBuf::from("~/.ssh/ed25519")),
            key_pass: None,
        },
        valid: true,
    };
    let m5 = Member {
        hostname: "redis".to_string(),
        ip: "192.168.1.2".to_string(),
        port: 22,
        user: "cx".to_string(),
        auth: Auth::PrivateKeyFile {
            key_file_path: Some(PathBuf::from("~/.ssh/ed25519")),
            key_pass: Some("pass".to_string()),
        },
        valid: true,
    };
    let group = Group {
        name: "groupname".to_string(),
        members: vec![m1, m2, m3, m4, m5],
        valid: true,
    };
    let server = Server {
        groups: vec![group],
    };

    serde_json::to_string_pretty(&server).unwrap()
}
