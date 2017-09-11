// joins?

extern crate rusqlite;

use std::fs::File;
use std::path::Path;
use std::error::Error;
use std::io::prelude::*;
use std::io::Cursor;
use std::io::BufReader;
use std::io::BufRead;
use core::convert::AsRef;

use Player;

const PLAYER_INIT_PATH: &str = "/home/rowan/documents/apps/nomic/autonomic_rust/db_init_player";
const PROPOSAL_INIT_PATH: &str = "/home/rowan/documents/apps/nomic/autonomic_rust/db_init_proposal";
const VOTE_INIT_PATH: &str = "/home/rowan/documents/apps/nomic/autonomic_rust/db_init_vote";
const RULE_INIT_PATH: &str = "/home/rowan/documents/apps/nomic/autonomic_rust/db_init_rule";
const TURN_INIT_PATH: &str = "/home/rowan/documents/apps/nomic/autonomic_rust/db_init_turn";
const INIT_PATHS: [&str; 5] = [
    PLAYER_INIT_PATH,
    PROPOSAL_INIT_PATH,
    VOTE_INIT_PATH,
    RULE_INIT_PATH,
    TURN_INIT_PATH];


pub fn init_db(conn: &rusqlite::Connection) {
    for file in INIT_PATHS.iter().map(|path| File::open(path).unwrap()) {
        let reader = BufReader::new(&file);
        for (_, line) in reader.lines().enumerate() {
            conn.execute(&line.unwrap()[..], &[]).unwrap();
        };
    };
}


pub fn print_player_table(conn: &rusqlite::Connection) {
    let mut stmt = conn.prepare("SELECT * FROM player").unwrap();
    let db_iter = stmt.query_map(&[], {
        |row| Player {
            id: row.get(0),
            username: row.get(1),
            points: row.get(2),
            timezone: row.get(3),
        }
    }).unwrap();
    for player in db_iter {
        println!("{:?}", player.unwrap());
    };
}


pub fn get_username_from_id(conn: &rusqlite::Connection, user_id: i32) -> Result<String,rusqlite::Error> {
    conn.query_row(&format!("SELECT username FROM player WHERE id='{}'", user_id)[..], &[], |row| row.get(0))
}


pub fn id_exists(conn: &rusqlite::Connection, table_name: &str, id: i32) -> bool {
    let maybe_row: Result<String,rusqlite::Error> =
        conn.query_row(&format!("SELECT id FROM {} WHERE id='{}'", table_name, id)[..], &[], |row| row.get(0));
    match maybe_row {
        Ok(_) => true,
        Err(_) => false,
    }
}


pub fn next_proposal_id(conn: &rusqlite::Connection) -> i32 {
    let mut stmt = conn.prepare("SELECT id FROM proposal").unwrap();
    let prop_ids = stmt.query_map(&[], |row| row.get(0)).unwrap().map(|opt_num| opt_num.unwrap());
    match prop_ids.max() {
        Some(num) => num,
        None => 300,
    }
}
