// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Commands for Bindle Cloud and Ours (handbook P15): starting with a
//! recovery kit, making and joining a shared space, inviting someone, and
//! keeping Ours in step. Keys live in the system keychain; everything that
//! leaves this device is sealed by `bindle-cloud` first.

use super::{announce, lock, CommandError, Reply};
use crate::{secrets, Db};
use bindle_cloud::client::Batch;
use bindle_cloud::codes::{self, Backup, BackupSpace, Invite};
use bindle_cloud::{CloudError, Key, Relay};
use bindle_core::Promise;
use bindle_store::cloud::{self, CloudAccount, Member, Space};
use chrono::{Duration, SecondsFormat, Utc};
use serde::Serialize;
use tauri::{AppHandle, Manager, State as Managed};

/// How often Ours is kept in step while Bindle runs.
const SYNC_EVERY: std::time::Duration = std::time::Duration::from_secs(30);

const RECOVERY: &str = "cloud-recovery-key";

fn space_secret(id: &str) -> String {
    format!("space-key-{id}")
}

impl From<CloudError> for CommandError {
    fn from(error: CloudError) -> Self {
        Self {
            message: error.to_string(),
        }
    }
}

fn plain(message: impl Into<String>) -> CommandError {
    CommandError {
        message: message.into(),
    }
}

fn relay_address(text: &str) -> Reply<String> {
    let text = text.trim().trim_end_matches('/');
    if text.starts_with("https://")
        || text.starts_with("http://localhost")
        || text.starts_with("http://127.0.0.1")
    {
        Ok(text.to_string())
    } else {
        Err(plain("the Bindle Cloud address starts with https://"))
    }
}

/// What Settings and Ours show about Bindle Cloud.
#[derive(Debug, Serialize)]
pub struct CloudStatus {
    pub account: CloudAccount,
    pub spaces: Vec<Space>,
}

#[tauri::command]
pub fn cloud_status(db: Managed<Db>) -> Reply<CloudStatus> {
    let conn = lock(&db)?;
    Ok(CloudStatus {
        account: cloud::cloud_account(&conn)?,
        spaces: cloud::spaces(&conn)?,
    })
}

/// Starts Bindle Cloud on this device: makes a recovery key, keeps it in
/// the keychain, and saves an empty backup on the relay. Returns the
/// recovery kit to write down.
#[tauri::command]
pub async fn cloud_start(app: AppHandle, relay: String, name: String) -> Reply<String> {
    let relay = relay_address(&relay)?;
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(plain("type the name others will see"));
    }
    let kit = Key::random();
    Relay::new(&relay)?
        .put_backup(
            &kit,
            &Backup {
                name: name.clone(),
                spaces: Vec::new(),
            },
        )
        .await?;
    secrets::set(RECOVERY, &kit.to_text()).map_err(plain)?;
    let db = app.state::<Db>();
    let conn = lock(&db)?;
    cloud::save_cloud_account(
        &conn,
        &CloudAccount {
            relay: Some(relay),
            name: Some(name),
            account: Some(kit.id()),
            device: Some(Key::random().id()),
            kit_saved_at: None,
        },
    )?;
    Ok(codes::recovery_text(&kit))
}

/// The recovery kit again, from this device's keychain.
#[tauri::command]
pub fn cloud_recovery_kit() -> Reply<String> {
    let kit = secrets::get(RECOVERY)
        .ok_or_else(|| plain("Bindle Cloud is not started on this device"))?;
    Ok(codes::recovery_text(
        &Key::from_text(&kit).map_err(|e| plain(e.to_string()))?,
    ))
}

/// Notes that the person saved their recovery kit.
#[tauri::command]
pub fn cloud_kit_saved(db: Managed<Db>) -> Reply<()> {
    let conn = lock(&db)?;
    let mut account = cloud::cloud_account(&conn)?;
    account.kit_saved_at = Some(Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true));
    Ok(cloud::save_cloud_account(&conn, &account)?)
}

/// Brings Bindle Cloud and every shared space back on a new device, from
/// the recovery kit.
#[tauri::command]
pub async fn cloud_restore(app: AppHandle, relay: String, kit: String) -> Reply<CloudStatus> {
    let relay = relay_address(&relay)?;
    let key = codes::read_recovery_text(&kit)
        .map_err(|_| plain("that recovery kit has a typo; check each group of four"))?;
    let backup = Relay::new(&relay)?.get_backup(&key).await?;
    secrets::set(RECOVERY, &key.to_text()).map_err(plain)?;
    for space in &backup.spaces {
        let space_key = Key::from_text(&space.key).map_err(|e| plain(e.to_string()))?;
        secrets::set(&space_secret(&space_key.id()), &space.key).map_err(plain)?;
    }
    {
        let db = app.state::<Db>();
        let conn = lock(&db)?;
        for space in &backup.spaces {
            let id = Key::from_text(&space.key)
                .map_err(|e| plain(e.to_string()))?
                .id();
            cloud::add_space(&conn, &id, &space.name, &space.relay, Utc::now())?;
        }
        cloud::save_cloud_account(
            &conn,
            &CloudAccount {
                relay: Some(relay),
                name: Some(backup.name.clone()),
                account: Some(key.id()),
                device: Some(Key::random().id()),
                kit_saved_at: Some(Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)),
            },
        )?;
    }
    sync_all(&app).await?;
    let db = app.state::<Db>();
    let conn = lock(&db)?;
    Ok(CloudStatus {
        account: cloud::cloud_account(&conn)?,
        spaces: cloud::spaces(&conn)?,
    })
}

/// Saves the list of spaces on the relay, sealed with the recovery key, so
/// the recovery kit can bring them back.
async fn save_backup(app: &AppHandle) -> Reply<()> {
    let kit = secrets::get(RECOVERY)
        .ok_or_else(|| plain("Bindle Cloud is not started on this device"))?;
    let kit = Key::from_text(&kit).map_err(|e| plain(e.to_string()))?;
    let (account, spaces) = {
        let db = app.state::<Db>();
        let conn = lock(&db)?;
        (cloud::cloud_account(&conn)?, cloud::spaces(&conn)?)
    };
    let relay = account
        .relay
        .clone()
        .ok_or_else(|| plain("Bindle Cloud is not started on this device"))?;
    let backup = Backup {
        name: account.name.unwrap_or_default(),
        spaces: spaces
            .iter()
            .filter_map(|s| {
                Some(BackupSpace {
                    key: secrets::get(&space_secret(&s.id))?,
                    name: s.name.clone(),
                    relay: s.relay.clone().unwrap_or_else(|| relay.clone()),
                })
            })
            .collect(),
    };
    Ok(Relay::new(&relay)?.put_backup(&kit, &backup).await?)
}

fn started(app: &AppHandle) -> Reply<CloudAccount> {
    let db = app.state::<Db>();
    let account = cloud::cloud_account(&*lock(&db)?)?;
    if account.relay.is_none() || account.account.is_none() {
        return Err(plain("start Bindle Cloud in Settings first"));
    }
    Ok(account)
}

/// Makes a new shared space.
#[tauri::command]
pub async fn ours_create(app: AppHandle, name: String) -> Reply<Space> {
    let account = started(&app)?;
    let name = if name.trim().is_empty() {
        "Ours".to_string()
    } else {
        name.trim().to_string()
    };
    let key = Key::random();
    secrets::set(&space_secret(&key.id()), &key.to_text()).map_err(plain)?;
    {
        let db = app.state::<Db>();
        let conn = lock(&db)?;
        cloud::add_space(
            &conn,
            &key.id(),
            &name,
            account.relay.as_deref().unwrap_or_default(),
            Utc::now(),
        )?;
    }
    save_backup(&app).await?;
    sync_all(&app).await?;
    let db = app.state::<Db>();
    let conn = lock(&db)?;
    cloud::spaces(&conn)?
        .into_iter()
        .find(|s| s.id == key.id())
        .ok_or_else(|| plain("the space was not saved"))
}

/// Makes an invite link for one person: it works once, for 7 days.
#[tauri::command]
pub async fn ours_invite(app: AppHandle, space_id: String) -> Reply<String> {
    let account = started(&app)?;
    let key = secrets::get(&space_secret(&space_id))
        .ok_or_else(|| plain("this device does not hold that space's key"))?;
    let (name, relay) = {
        let db = app.state::<Db>();
        let conn = lock(&db)?;
        let space = cloud::spaces(&conn)?
            .into_iter()
            .find(|s| s.id == space_id)
            .ok_or_else(|| plain("that space is not here"))?;
        (
            space.name,
            space.relay.or(account.relay.clone()).unwrap_or_default(),
        )
    };
    let secret = Key::random();
    let invite = Invite {
        space_key: key,
        space_name: name,
        from: account.name.unwrap_or_default(),
        relay: relay.clone(),
    };
    Relay::new(&relay)?.put_invite(&secret, &invite).await?;
    Ok(codes::invite_link(&secret, &relay))
}

/// Joins a shared space from an invite link. Bindle Cloud must be started
/// first (with a name), on any relay.
#[tauri::command]
pub async fn ours_join(app: AppHandle, link: String) -> Reply<Space> {
    let account = started(&app)?;
    let (secret, relay) =
        codes::read_invite_link(&link).map_err(|_| plain("that is not a Bindle invite link"))?;
    let relay = relay.or(account.relay.clone()).unwrap_or_default();
    let invite = Relay::new(&relay)?.take_invite(&secret).await?;
    let key = Key::from_text(&invite.space_key).map_err(|e| plain(e.to_string()))?;
    secrets::set(&space_secret(&key.id()), &key.to_text()).map_err(plain)?;
    {
        let db = app.state::<Db>();
        let conn = lock(&db)?;
        cloud::add_space(
            &conn,
            &key.id(),
            &invite.space_name,
            &invite.relay,
            Utc::now(),
        )?;
    }
    save_backup(&app).await?;
    sync_all(&app).await?;
    let db = app.state::<Db>();
    let conn = lock(&db)?;
    cloud::spaces(&conn)?
        .into_iter()
        .find(|s| s.id == key.id())
        .ok_or_else(|| plain("the space was not saved"))
}

/// Puts a promise in a shared space, or back in Mine.
#[tauri::command]
pub fn ours_share(
    app: AppHandle,
    db: Managed<Db>,
    promise_id: String,
    space_id: Option<String>,
) -> Reply<()> {
    {
        let conn = lock(&db)?;
        let account = cloud::cloud_account(&conn)?.account.unwrap_or_default();
        cloud::share(&conn, &promise_id, space_id.as_deref(), &account)?;
    }
    announce(&app);
    Ok(())
}

/// Jots a promise straight into a shared space. It waits in Sort like any other.
#[tauri::command]
pub fn ours_jot(app: AppHandle, db: Managed<Db>, space_id: String, text: String) -> Reply<Promise> {
    let promise = super::jot_down(app.clone(), db.clone(), text)?;
    ours_share(app, db, promise.id.clone(), Some(space_id))?;
    Ok(promise)
}

/// A promise in Ours, with who added it.
#[derive(Debug, Serialize)]
pub struct SharedPromise {
    pub promise: Promise,
    /// The name of whoever added it, or None when it was you.
    pub added_by: Option<String>,
    pub done_by: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OursView {
    pub space: Space,
    pub members: Vec<Member>,
    pub me: Option<String>,
    pub open: Vec<SharedPromise>,
    pub kept_lately: Vec<SharedPromise>,
    pub kept_this_month: u32,
}

/// Everything the Ours screen shows for one space.
#[tauri::command]
pub fn ours_view(db: Managed<Db>, space_id: String) -> Reply<OursView> {
    let conn = lock(&db)?;
    let space = cloud::spaces(&conn)?
        .into_iter()
        .find(|s| s.id == space_id)
        .ok_or_else(|| plain("that space is not here"))?;
    let members = cloud::members(&conn, &space_id)?;
    let me = cloud::cloud_account(&conn)?.account;
    let name_of = |account: Option<String>| -> Option<String> {
        let account = account?;
        if Some(&account) == me.as_ref() {
            return None;
        }
        Some(
            members
                .iter()
                .find(|m| m.account == account)
                .map_or_else(|| "someone".into(), |m| m.name.clone()),
        )
    };
    let month_ago = (Utc::now() - Duration::days(30)).to_rfc3339_opts(SecondsFormat::Millis, true);
    let (mut open, mut kept_lately, mut kept_this_month) = (Vec::new(), Vec::new(), 0);
    for row in cloud::shared_promises(&conn, &space_id)? {
        let shared = SharedPromise {
            added_by: name_of(row.created_by),
            done_by: name_of(row.done_by),
            promise: row.promise,
        };
        if shared.promise.state == bindle_core::State::Done {
            if shared
                .promise
                .closed_at
                .as_deref()
                .is_some_and(|c| c >= month_ago.as_str())
            {
                kept_this_month += 1;
                kept_lately.push(shared);
            }
        } else {
            open.push(shared);
        }
    }
    kept_lately.sort_by(|a, b| b.promise.closed_at.cmp(&a.promise.closed_at));
    kept_lately.truncate(5);
    Ok(OursView {
        space,
        members,
        me,
        open,
        kept_lately,
        kept_this_month,
    })
}

/// Leaves a space on this device. Its promises stay here, in Mine.
#[tauri::command]
pub async fn ours_leave(app: AppHandle, space_id: String) -> Reply<()> {
    {
        let db = app.state::<Db>();
        let conn = lock(&db)?;
        cloud::leave(&conn, &space_id)?;
    }
    let _ = secrets::forget(&space_secret(&space_id));
    let _ = save_backup(&app).await;
    announce(&app);
    Ok(())
}

/// Keeps every space in step now.
#[tauri::command]
pub async fn cloud_sync(app: AppHandle) -> Reply<CloudStatus> {
    sync_all(&app).await?;
    let db = app.state::<Db>();
    let conn = lock(&db)?;
    Ok(CloudStatus {
        account: cloud::cloud_account(&conn)?,
        spaces: cloud::spaces(&conn)?,
    })
}

/// Sends what changed here and takes what changed elsewhere, for each space.
pub(crate) async fn sync_all(app: &AppHandle) -> Reply<()> {
    let db = app.state::<Db>();
    let (account, spaces) = {
        let conn = lock(&db)?;
        (cloud::cloud_account(&conn)?, cloud::spaces(&conn)?)
    };
    let (Some(device), Some(me)) = (account.device.clone(), account.account.clone()) else {
        return Ok(());
    };
    let name = account.name.clone().unwrap_or_default();
    let mut changed = false;
    for space in spaces {
        let Some(key) =
            secrets::get(&space_secret(&space.id)).and_then(|k| Key::from_text(&k).ok())
        else {
            continue;
        };
        let relay = Relay::new(
            space
                .relay
                .as_deref()
                .or(account.relay.as_deref())
                .unwrap_or_default(),
        )?;
        let outbox = {
            let mut conn = lock(&db)?;
            cloud::collect_changes(&mut conn, &space.id, &device, &me, &name, Utc::now())?;
            cloud::outbox(&conn, &space.id)?
        };
        for (id, json) in outbox {
            let ops = serde_json::from_str(&json).map_err(|e| plain(e.to_string()))?;
            let plain_batch = serde_json::to_vec(&Batch { ops }).expect("a batch serialises");
            relay.push(&key, &plain_batch).await?;
            cloud::sent(&*lock(&db)?, id)?;
        }
        let after = cloud::pulled(&*lock(&db)?, &space.id)?;
        let batches = relay.pull(&key, after).await?;
        let mut conn = lock(&db)?;
        for (seq, bytes) in &batches {
            if let Ok(batch) = serde_json::from_slice::<Batch>(bytes) {
                changed |= cloud::apply_changes(&mut conn, &space.id, &batch.ops, Utc::now())? > 0;
            }
            cloud::set_pulled(&conn, &space.id, *seq, Utc::now())?;
        }
        if batches.is_empty() {
            cloud::set_pulled(&conn, &space.id, after, Utc::now())?;
        }
    }
    if changed {
        announce(app);
    }
    Ok(())
}

/// Keeps Ours in step in the background while Bindle runs.
pub fn start(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(SYNC_EVERY).await;
            if let Err(error) = sync_all(&app).await {
                eprintln!("Bindle: Ours not synced this time: {}", error.message);
            }
        }
    });
}
