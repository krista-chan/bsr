use adb::Adb;
use dotenv::dotenv;
use zip::ZipArchive;
use std::env::var;
use std::fs::File;
use std::io::BufReader;
use std::str::Split;
use twitchchat::connector::tokio::Connector;
use twitchchat::messages::Commands;
use twitchchat::runner::AsyncRunner;
use twitchchat::twitch::Capability::*;
use twitchchat::{PrivmsgExt, Status, UserConfig};

use crate::bsaber::{get_map_info, download_map_zip};

mod adb;
mod bsaber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let oauth = var("OAUTH_TOKEN").unwrap();
    let nick = var("BOT_USERNAME").unwrap();
    let room = var("CHANNEL_NAME").unwrap();

    let mut adb = Adb::new();
    adb.connect_abd();

    let config = UserConfig::builder()
        .capabilities(&[Tags, Commands, Membership])
        .name(&nick)
        .token(oauth)
        .build()?;

    let mut runner = connect(&config, &[room]).await.unwrap();

    while let Ok(next) = runner.next_message().await {
        match next {
            Status::Message(Commands::Privmsg(pm)) => {
                let author = pm.name();
                let mut content = pm.data().to_string();

                println!("[BsrBot] {author} sent {content}");

                if content.remove(0) != '!' {
                    continue;
                }

                let args = content.trim().split_whitespace().collect::<Vec<_>>();

                match args[0] {
                    "ping" => {
                        runner.writer().reply(&pm, "Pong!").unwrap();
                    }
                    "quit" => {
                        if author != "dice2x5" {
                            runner
                                .writer()
                                .reply(&pm, "You do not have permissions to run this command.")
                                .unwrap();
                            continue;
                        }

                        runner.writer().say(&pm, "Shutting down...").unwrap();
                        runner.quit_handle().notify().await;
                    }
                    "bsr" => {
                        if args.len() <= 1 {
                            runner
                                .writer()
                                .reply(
                                    &pm,
                                    "You must provide a song ID or URL from https://bsaber.com.",
                                )
                                .unwrap();
                        }
                        let id = args[1];

                        let songid = if id.starts_with("https") {
                            let clean_url = id.replace("https://", "");
                            let url_split = clean_url.trim_end_matches('/').split('/');

                            if let Some(id) = validate_url(url_split) {
                                id
                            } else {
                                runner.writer().reply(&pm, "Invalid bsaber URL.").unwrap();
                                continue;
                            }
                        } else {
                            if !id.chars().all(|c| c.is_ascii_alphanumeric()) {
                                runner
                                    .writer()
                                    .reply(&pm, "Invalid bsaber song ID.")
                                    .unwrap();
                                continue;
                            }

                            id.to_owned()
                        };

                        let mapinfo = tokio::task::spawn_blocking(|| get_map_info(songid)).await.unwrap();
                        let name = mapinfo.name;
                        let url = mapinfo.url;
                        let hash = mapinfo.hash;

                        runner.writer().say(&pm, &format!("@{author} requested {name} ({url})")).unwrap();

                        download_map_zip(url, hash.clone());
                        let inner = File::open(&format!("tmp/{hash}.zip")).unwrap();
                        let reader = BufReader::new(inner);
                        let mut zip = ZipArchive::new(reader).unwrap();

                        zip.extract(&format!("tmp/{hash}")).unwrap();

                        adb.push_map(hash, name);
                    }
                    _ => (),
                }
            }

            Status::Quit | Status::Eof => break,
            Status::Message(..) => continue,
        }
    }

    Ok(())
}

async fn connect(user_config: &UserConfig, channels: &[String]) -> anyhow::Result<AsyncRunner> {
    let connector = Connector::twitch()?;

    println!("[BsrBot] Connecting to twitch...");

    let mut runner = AsyncRunner::connect(connector, user_config).await?;
    println!("[BsrBot] Connected to twitch!");

    for channel in channels {
        println!("[BsrBot] Attempting to join channel '{}'", channel);
        runner.join(channel).await?;
        println!("[BsrBot] Joined channel '{}'", channel);
    }

    Ok(runner)
}

fn validate_url(params: Split<char>) -> Option<String> {
    if let ["bsaber.com", "songs", id] = params.collect::<Vec<_>>()[..] {
        Some(id.into())
    } else {
        None
    }
}
