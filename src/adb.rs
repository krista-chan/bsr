use json::{object, parse};
use std::fs::File;
use std::io::Read;
use std::process::Command;
use std::{env::var, fs::OpenOptions};

#[derive(Debug, Clone)]
pub struct Adb {
    ip: String,
    adb_path: String,
    songreq_path: String,
}

impl Adb {
    pub fn new() -> Self {
        Adb {
            ip: String::new(),
            adb_path: var("ADB_BINARY").unwrap(),
            songreq_path: var("SONGREQ_PATH").unwrap(),
        }
    }

    fn get_ip(&mut self) {
        let out = Command::new(&*self.adb_path)
            .args(["shell", "ip", "addr", "show", "wlan0"])
            .output()
            .unwrap();

        let stdout = out
            .stdout
            .into_iter()
            .map(|c| c as char)
            .collect::<String>();
        let mut split = stdout.split_ascii_whitespace();

        split.position(|s| s == "inet");
        let ip = split
            .next()
            .map(|ip| ip.split('/').collect::<Vec<_>>()[0])
            .unwrap();

        println!("[BsrBot] Quest IP is {ip}");

        self.ip = ip.to_owned();
    }

    pub fn connect_abd(&mut self) {
        self.get_ip();
        println!("[BsrBot : adb] Launching ADB tcp server");
        Command::new(&*self.adb_path)
            .args(["tcpip", "5555"])
            .output()
            .unwrap();

        println!("[BsrBot : adb] Connecting to ADB server");
        Command::new(&*self.adb_path)
            .args(["connect", &format!("{}:5555", self.ip)])
            .output()
            .unwrap();
    }

    pub fn push_map(&mut self, hash: String, name: String) {
        println!("[BsrBot : adb] Uploading map to quest");
        Command::new(&*self.adb_path)
            .args([
                "-s",
                &format!("{}:5555", self.ip),
                "push",
                &format!("tmp/{hash}"),
                &format!(
                    "/sdcard/ModData/com.beatgames.beatsaber/Mods/SongLoader/CustomLevels/{hash}"
                ),
            ])
            .output()
            .unwrap();

        println!("[BsrBot : adb] Map {name} uploaded!");

        self.update_playlist(hash, name);
    }

    fn update_playlist(&mut self, hash: String, name: String) {
        println!("[BsrBot : adb] Downloading playlist.json for song requests");
        Command::new(&*self.adb_path)
            .args([
                "-s",
                &format!("{}:5555", self.ip),
                "pull",
                &*self.songreq_path,
                "tmp/songreq.json",
            ])
            .output()
            .unwrap();

        let mut buf = Vec::new();
        File::open("tmp/songreq.json")
            .unwrap()
            .read_to_end(&mut buf)
            .unwrap();
        let mut json = parse(&buf.into_iter().map(|b| b as char).collect::<String>()).unwrap();

        json["songs"]
            .push(object! { hash: hash, songName: name })
            .unwrap();

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open("tmp/songreq.json")
            .unwrap();
        json.write(&mut file).unwrap();

        println!("[BsrBot : adb] Reuploading playlist.json");
        Command::new(&*self.adb_path)
            .args([
                "-s",
                &format!("{}:5555", self.ip),
                "push",
                "tmp/songreq.json",
                &*self.songreq_path,
            ])
            .output()
            .unwrap();
    }
}

impl Drop for Adb {
    fn drop(&mut self) {
        Command::new(&*self.adb_path)
            .args(["disconnect", &format!("{}:5555", self.ip)])
            .output()
            .unwrap();
    }
}
