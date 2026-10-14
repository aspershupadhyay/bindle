// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Commands for the Board (v2 design, Board; handbook P14).

use super::{announce, lock, Reply};
use crate::Db;
use bindle_store::board::{self, BoardView};
use tauri::{AppHandle, State as Managed};

/// Work promises in their lanes.
#[tauri::command]
pub fn board_view(db: Managed<Db>) -> Reply<BoardView> {
    Ok(board::board_view(&*lock(&db)?, super::home::week_start())?)
}

/// Moves an open promise into Making, or back out of it.
#[tauri::command]
pub fn set_making(app: AppHandle, db: Managed<Db>, id: String, making: bool) -> Reply<()> {
    board::set_making(&*lock(&db)?, &id, making)?;
    announce(&app);
    Ok(())
}
