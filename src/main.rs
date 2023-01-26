use serde::Serialize;
use std::{
    env, error, fs,
    io::{Read, Write},
    path::PathBuf,
};
use xdg::BaseDirectories;

use freedesktop_desktop_entry::{default_paths, DesktopEntry, Iter};

const DEFAULT_ICON: &str = "wayland";
const ICON_SIZE: u16 = 48;

struct App {
    name: Option<String>,
    icon: Option<String>,
    exec: String,
}

// TODO maybe just use builder pattern
// same as App, but uses an icon path, and all fields should have a value
#[derive(Serialize)]
struct EwwAppEntry {
    name: String,
    icon: PathBuf,
    exec: String,
}

enum ErrorAppConversion {
    ErrDefaultIconNotFound,
    ErrMissingAppName,
}

impl TryFrom<App> for EwwAppEntry {
    type Error = ErrorAppConversion;
    fn try_from(app: App) -> Result<Self, Self::Error> {
        use ErrorAppConversion::*;
        let find_icon = |name| {
            freedesktop_icons::lookup(name)
                .with_size(ICON_SIZE)
                .with_cache()
                .find()
        };

        // fallbacks to DEFAULT_ICON, errors out if DEFAULT_ICON is not found
        let icon = if let Some(name) = app.icon {
            find_icon(&name).or_else(|| find_icon(DEFAULT_ICON))
        } else {
            find_icon(DEFAULT_ICON)
        };
        let icon = icon.ok_or(ErrDefaultIconNotFound)?;

        Ok(EwwAppEntry {
            name: app.name.ok_or(ErrMissingAppName)?,
            icon,
            exec: app.exec,
        })
    }
}

fn main() {
    let args = env::args().skip(1).take(1).collect::<Vec<_>>();
    let arg = args.get(0).and_then(|s| Some(s.as_str()));
    match arg {
        None | Some("-l") => list_apps(),
        _ => todo!(""),
    }
    // println!("{}", serde_json::to_string(&produce_eww_entries()).unwrap());
}

fn list_apps() {
    if let Ok(fname) = get_cache_path() {
        if let Ok(mut file) = fs::File::open(fname) {
            let mut buf = vec![];
            file.read_to_end(&mut buf).expect("TODO");
            let data = std::str::from_utf8(&buf).expect("TODO");
            println!("{}", data);
            return;
        }
    }

    let entries = produce_eww_entries();
    let json = serde_json::to_string(&entries).expect("TODO");
    println!("{}", json);
    write_cache(&entries).expect("TODO");
}

/// finds all apps, skips apps that are missing an Exec value
fn get_apps() -> Vec<App> {
    Iter::new(default_paths())
        .filter_map(|fpath| {
            // maybe log errors
            let bytes = fs::read_to_string(&fpath).ok()?;
            let entry = DesktopEntry::decode(&fpath, &bytes).ok()?;
            Some(App {
                // TODO use file name as fallback
                name: entry.name(None).and_then(|x| Some(x.to_string())),
                icon: entry
                    .icon()
                    .and_then(|x| Some(x.to_string()))
                    .or(Some(DEFAULT_ICON.to_string())),
                exec: entry.exec()?.to_string(),
            })
        })
        .collect()
}

fn produce_eww_entries() -> Vec<EwwAppEntry> {
    get_apps()
        .into_iter()
        .filter_map(|app| EwwAppEntry::try_from(app).ok())
        .collect()
}

fn write_cache(entries: &Vec<EwwAppEntry>) -> Result<(), Box<dyn error::Error>> {
    let data = serde_json::to_string(entries)?;
    let fpath = get_cache_path()?;
    // let mut file = fs::File::open(fpath)?;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(fpath)?;

    file.write_all(data.as_bytes())?;

    Ok(())
}

fn get_cache_path() -> Result<PathBuf, Box<dyn error::Error>> {
    Ok(BaseDirectories::with_prefix("eww-dock")?.place_cache_file("eww-dock-cache.json")?)
}
