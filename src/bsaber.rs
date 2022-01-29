use std::{fmt::Display, fs::File, io::Write};

use json::parse;
use reqwest::blocking::Client;

pub fn get_map_info(id: String) -> MapInfo {
    println!("[BsrBot : Bsaber] Getting information for map {id}");
    let client = Client::new();
    let url = format!("https://api.beatsaver.com/maps/id/{id}");

    let res = client.get(url).send().unwrap().text().unwrap();
    let json = parse(&res).unwrap();

    let version = &json["versions"][0];
    let dl_url = &version["downloadURL"];
    let hash = &version["hash"];
    let song_name = &json["metadata"]["songName"];

    MapInfo {
        url: dl_url.as_str().unwrap().to_string(),
        name: song_name.as_str().unwrap().to_string(),
        hash: hash.as_str().unwrap().to_string(),
    }
}

pub fn download_map_zip<T>(url: T, hash: T)
where
    T: AsRef<str> + Display,
{
    let mut file = File::create(format!("tmp/{hash}.zip")).unwrap();
    let client = Client::new();

    println!("[BsrBot] Downloading beatmap...");
    let zip = client.get(url.as_ref()).send().unwrap().bytes().unwrap();

    file.write_all(&*zip).unwrap();
    println!("[BsrBot] Beatmap downloaded!");
}

#[derive(Debug)]
pub struct MapInfo {
    pub url: String,
    pub name: String,
    pub hash: String,
}
