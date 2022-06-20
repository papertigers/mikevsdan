mod config;
mod models;

use std::{
    cmp::Reverse,
    fs::{self, File, OpenOptions},
    io::Write,
    os::unix::fs::PermissionsExt,
    path::Path,
};

use anyhow::Result;
use config::Config;
use minijinja::Environment;
use models::*;
use reqwest::Url;
use serde::Serialize;

static BASE_URL: &str = "https://gamesheet.app";

#[derive(Debug, Serialize)]
struct Player {
    name: String,
    stats: Stats,
    avatar: String,
}

fn update_file<P, N, F>(output_dir: P, file_name: N, mode: u32, func: F) -> Result<()>
where
    P: AsRef<Path>,
    N: AsRef<str>,
    F: Fn(&mut File) -> Result<()>,
{
    let dir = output_dir.as_ref();
    let file = file_name.as_ref();
    let mut from_path = dir.join(file);
    from_path.set_extension("temp");
    let to_path = dir.join(file);

    let mut temp_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&from_path)?;

    let mut perms = temp_file.metadata()?.permissions();
    perms.set_mode(mode);
    temp_file.set_permissions(perms)?;

    func(&mut temp_file)?;
    std::fs::rename(from_path, to_path).map_err(|e| e.into())
}

fn update_templated_file<F: AsRef<str>, P: AsRef<Path>>(
    file: F,
    output_dir: P,
    mode: u32,
    contents: &[u8],
) -> Result<()> {
    let file = file.as_ref();
    update_file(output_dir, file, mode, |f| {
        f.write_all(contents).map_err(|e| e.into())
    })
}

fn get_player_stats(player: &str) -> Result<Stats> {
    let mut url = Url::parse(BASE_URL).unwrap();
    url.set_path(&format!("api/stats/v1/players/{player}/career-stats"));
    let json: ApiData = reqwest::blocking::get(url)?.json()?;

    for d in json.data {
        match d {
            PlayerData::PlayerTeamSeasonStats { stats } => return Ok(stats),
            _ => continue,
        }
    }

    Err(anyhow::anyhow!("Stats for {player} missing"))
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let program = &args[0].clone();
    let brief = format!("Usage: {} [options] -c CONFIG", program);

    let mut opts = getopts::Options::new();
    opts.reqopt("c", "", "config file", "CONFIG");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => return Err(anyhow::anyhow!("{}\n{}", e, opts.usage(&brief))),
    };

    let config = Config::from_file(matches.opt_str("c").unwrap())?;
    let mut players = Vec::with_capacity(config.players.len());

    // Get stats for all of the competing players
    for player in config.players {
        players.push(Player {
            name: player.name.to_owned(),
            stats: get_player_stats(&player.uuid)?,
            avatar: player.avatar.to_owned(),
        });
    }

    // Make sure the player leading in points is first.
    players.sort_by_key(|p| Reverse(p.stats.points));

    // Create the minijina template
    let mut env = Environment::new();
    let template = fs::read_to_string(config.template.path)?;
    env.add_template(&config.template.name, &template)?;

    // Render the minijina template
    let json = serde_json::json!({ "players": players });
    let template = env.get_template(&config.template.name)?;
    let contents = template.render(&json)?;

    // Write out the updated file contents
    update_templated_file(
        &config.template.name,
        &config.output_dir,
        config.template.mode,
        contents.as_bytes(),
    )?;

    Ok(())
}
