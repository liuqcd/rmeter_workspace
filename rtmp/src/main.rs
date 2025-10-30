use std::path::PathBuf;

use serde_derive::{Deserialize, Serialize};
use serde_json;

fn main() {
    // der();
    ser();
}

fn der() {
    // let json_str = r#"{"groups":[{"name":"groupname","members":[{"hostname":"redis","ip":"192.168.1.2","port":22,"user":"cx","auth":{"PrivateKeyFile":{"key_file_path":"~/.ssh/ed25519","key_pass":null}},"valid":true}],"valid":true}]}"#;
    // let json_str = r#"{"groups":[{"name":"groupname","members":[{"hostname":"redis","ip":"192.168.1.2","port":22,"user":"cx","auth":{"PrivateKeyFile":{"key_file_path":"~/.ssh/ed25519"}},"valid":true}],"valid":true}]}"#;
    let json_str = r#"{"groups":[{"name":"groupname","members":[{"hostname":"redis","ip":"192.168.1.2","port":22,"user":"cx","auth":{"PrivateKeyFile":{}},"valid":true}],"valid":true}]}"#;
    let server: Server = serde_json::from_str(&json_str).unwrap();
    println!("{:?}", server);
}

fn ser() {
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

    println!("{}", serde_json::to_string(&server).unwrap());
    println!("Hello, world!");
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
// pub enum AuthMethod {
//     Password(String),
//     PrivateKey {
//         key_data: String,
//         key_pass: Option<String>,
//     },
//     PrivateKeyFile {
//         key_file_path: Option<PathBuf>,
//         key_pass: Option<String>,
//     },
//     PublicKeyFile {
//         key_file_path: Option<PathBuf>,
//     },
//     Agent,
// }

/// 代表整个json配置文件
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Server {
    // #[serde(flatten)]
    groups: Vec<Group>,
}

/// 多行连接信息可组成一组
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Group {
    name: String,
    members: Vec<Member>,
    valid: bool,
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

// #[derive(Debug, Clone)]
// pub struct ClientInfo {
//     info: Vec<Info>,
// }

// #[derive(Debug, Clone)]
// pub enum Info {
//     SSH {
//         hostname: String,
//         ip: String,
//         port: u16,
//         username: String,
//         password: String,
//         groupname: String,
//     },
//     // RSA {
//     //     hostname: String,
//     //     ip: String,
//     //     port: u16,
//     //     username: String,
//     //     privatekey: Option<String>,
//     //     groupname: String,
//     // },
// }
