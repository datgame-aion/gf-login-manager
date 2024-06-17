#![windows_subsystem = "windows"]
slint::include_modules!();
use std::{os::windows::process::CommandExt, process::Command};

use anyhow::Result;
use directories::BaseDirs;
use rusqlite::{named_params, Connection};
use slint::{ModelRc, SharedString, Weak};

const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Debug)]
struct Login {
    display_name: String,
    creation_utc: i64,
    value: String,
    expires_utc: i64,
    last_access_utc: i64,
}

fn open_gf_db() -> Result<Connection> {
    let base_dir = BaseDirs::new().ok_or(anyhow::anyhow!("Can't create Base dir"))?;
    let cookies_path = base_dir
        .cache_dir()
        .join(r"Gameforge4d\GameforgeClient\webcache\Cookies");
    Ok(Connection::open(cookies_path)?)
}

fn open_manager_db() -> Result<Connection> {
    let base_dir = BaseDirs::new().ok_or(anyhow::anyhow!("Can't create Base dir"))?;
    let cookies_path = base_dir.data_dir().join(r"gf-login-manager.db");
    let conn = Connection::open(cookies_path)?;

    conn.execute(include_str!("create_db.sql"), [])?;

    Ok(conn)
}

/// gets the gf-token-production cookie
/// and saves it to the manager db
fn get_cookie(name: String, ui: Weak<MainWindow>) -> Result<Login> {
    let conn = open_gf_db()?;
    let manager_db = open_manager_db()?;

    let token: Login = conn.query_row(include_str!("get_token.sql"), [], |row| {
        Ok(Login {
            display_name: name,
            creation_utc: row.get(0)?,
            value: row.get(1)?,
            expires_utc: row.get(2)?,
            last_access_utc: row.get(3)?,
        })
    })?;

    manager_db.execute(
        include_str!("add_to_manager_db.sql"),
        named_params! {
        ":display_name": token.display_name,
        ":creation_utc": token.creation_utc,
        ":value": token.value,
        ":expires_utc": token.expires_utc,
        ":last_access_utc": token.last_access_utc,
        },
    )?;
    try_update_ui(ui);

    Ok(token)
}

/// kill the gfclient.exe
/// sets the gf-token-production cookie
/// starts gfclient.exe again
fn set_cookie(name: &str) -> Result<()> {
    let _ = Command::new("taskkill")
        .args(&["/IM", "gfclient.exe", "/F"])
        .creation_flags(CREATE_NO_WINDOW)
        .status();
    let conn = open_gf_db()?;
    let manager_db = open_manager_db()?;

    let token: Login = manager_db.query_row(
        include_str!("select_token_manager.sql"),
        &[(":display_name", name)],
        |row| {
            Ok(Login {
                display_name: row.get(0)?,
                creation_utc: row.get(1)?,
                value: row.get(2)?,
                expires_utc: row.get(3)?,
                last_access_utc: row.get(4)?,
            })
        },
    )?;

    // this insert the a new if cookie if none exist
    let _ = conn.execute(
        include_str!("set_token.sql"),
        named_params! {
        ":creation_utc": token.creation_utc,
        ":value": token.value,
        ":expires_utc": token.expires_utc,
        ":last_access_utc": token.last_access_utc,
        },
    );

    // this updates the cookie with the new token
    conn.execute(
        "UPDATE cookies set value = (:value) where name='gf-token-production'",
        named_params! {
        ":value": token.value,
        },
    )?;

    let _ = Command::new(r"C:\Program Files (x86)\GameforgeClient\gfclient.exe").spawn();
    Ok(())
}

fn try_update_ui(ui: Weak<MainWindow>) {
    let _ = ui.upgrade_in_event_loop(|ui| {
        if let Ok(names) = get_names() {
            ui.set_logins(names);
        }
    });
}

fn delete(name: &str, ui: Weak<MainWindow>) -> Result<()> {
    let manager_db = open_manager_db()?;

    let _ = manager_db.execute(
        "DELETE from cookies where display_name=(:name)",
        &[(":name", name)],
    )?;

    try_update_ui(ui);

    Ok(())
}

fn logout() -> Result<()> {
    let _ = Command::new("taskkill")
        .args(&["/IM", "gfclient.exe", "/F"])
        .creation_flags(CREATE_NO_WINDOW)
        .status();
    let conn = open_gf_db()?;

    // this updates the cookie with the new token
    let _ = conn.execute("DELETE from cookies where name='gf-token-production'", [])?;

    let _ = Command::new(r"C:\Program Files (x86)\GameforgeClient\gfclient.exe").spawn();
    Ok(())
}

fn get_names() -> Result<ModelRc<SharedString>> {
    let manager_db = open_manager_db()?;
    let mut stmt = manager_db.prepare("SELECT display_name FROM cookies")?;
    let names = stmt
        .query_map([], |row| Ok(row.get(0)?))?
        .filter_map(|x| x.ok())
        .collect::<Vec<String>>();
    let names = names
        .into_iter()
        .map(|x| SharedString::from(x))
        .collect::<Vec<_>>();

    Ok(ModelRc::new(slint::VecModel::from(names)))
}

fn main() -> Result<()> {
    let window = MainWindow::new()?;

    let ui = window.as_weak();
    let ui1 = window.as_weak();
    window.set_logins(get_names()?);
    window.on_clicked(|name| {
        println!("{name} {:?}", set_cookie(&name));
    });
    window.on_create(move |name| {
        let _ = get_cookie(name.into(), ui.clone());
    });
    window.on_clicked_delete(move |name| println!("{:?}", delete(&name, ui1.clone())));
    window.on_logout(|| {
        println!("{:?}", logout());
    });

    window.run()?;
    Ok(())
}
