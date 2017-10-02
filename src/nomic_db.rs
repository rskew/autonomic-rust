// joins?

extern crate rusqlite;

use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;

use Player;
use proposal::interpreter::Mutability;
use proposal::Proposal;

const PLAYER_INIT_PATH: &str =
    "/home/rowan/documents/apps/autonomic/db_init_player";
const PROPOSAL_INIT_PATH: &str =
    "/home/rowan/documents/apps/autonomic/db_init_proposal";
const VOTE_INIT_PATH: &str =
    "/home/rowan/documents/apps/autonomic/db_init_vote";
const RULE_INIT_PATH: &str =
    "/home/rowan/documents/apps/autonomic/db_init_rule";
const TURN_INIT_PATH: &str =
    "/home/rowan/documents/apps/autonomic/db_init_turn";
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
            slack_id: row.get(1),
            username: row.get(2),
            points: row.get(3),
            timezone: row.get(4),
        }
    }).unwrap();
    for player in db_iter {
        println!("{:?}", player.unwrap());
    };
}


pub fn id_exists(conn: &rusqlite::Connection, table_name: &str, id: i32)
        -> bool {
    let maybe_row: Result<String,rusqlite::Error> =
        conn.query_row(&format!("SELECT id FROM {} WHERE id='{}'",
                                table_name,
                                id)[..],
                       &[],
                       |row| row.get(0)
                       );
    match maybe_row {
        Ok(_) => true,
        Err(_) => false,
    }
}


pub fn last_proposal_id(conn: &rusqlite::Connection)
        -> Option<i32> {
    let mut int:i32 = 0;
    match conn.query_row("SELECT id FROM proposal ORDER BY id DESC LIMIT 1",
                       &[],
                       |row| { int = row.get(0) }
                       ) {
        Ok(_) => Some(int),
        Err(_) => None,
    }
}


pub fn check_prop_mutability(conn: &rusqlite::Connection, prop_id: i32)
    -> Result<Mutability,String> {
    let maybe_row: Result<String,rusqlite::Error> =
        conn.query_row(&format!("SELECT mutable FROM rule WHERE id='{}'",
                                prop_id)[..],
                       &[],
                       |row| row.get(0)
                       );
    match maybe_row {
        Ok(s) => Ok(Mutability::from_str(&s[..])
                    .expect("The mutability value in the db is not \
                            'mutable' or 'mmutable' wtf")),
        Err(_) => Err(String::from(
                "That proposal doesn't seem to be a current \
                rule, can't transmute that!"
                )),
    }
}


pub fn is_current_rule(conn: &rusqlite::Connection, prop_id: i32) -> bool {
    let maybe_row: Result<String,rusqlite::Error> =
        conn.query_row(&format!("SELECT 1 FROM rule WHERE id='{}'",
                                prop_id)[..],
                       &[],
                       |row| row.get(0)
                       );
    match maybe_row {
        Ok(_) => true,
        Err(_) => false,
    }
}


pub fn get_username(conn: &rusqlite::Connection, user_id: &str)
        -> Result<String,String> {
    let maybe_row: Result<String,rusqlite::Error> =
        conn.query_row(&format!("SELECT 1 FROM player WHERE slack_id='{}'",
                                user_id)[..],
                                &[],
                                |row| row.get(0));
    match maybe_row {
        Ok(username) => Ok(username),
        Err(_) => Err(format!("No such user for id {}", user_id)),
    }
}

pub fn register_proposal(conn: &rusqlite::Connection, proposal: &Proposal)
        -> Result<String,rusqlite::Error> {
    debug!("registering proposal: {:?}", &proposal);
    conn.execute("INSERT INTO proposal
                  (id, creator_id, timestamp_proposed, text) \
                  VALUES (?1, ?2, ?3, ?4)",
                  &[&proposal.id,
                    &proposal.creator_id,
                    &proposal.timestamp_proposed,
                    &proposal.text])?;
    Ok(String::from("Proposal registered:"))
}


pub fn get_current_proposal(conn: &rusqlite::Connection)
        -> Option<Proposal> {
    // Set this to get the current turn and get the proposal attached to it
    let mut prop_id: i32 = 300;
    match conn.query_row("SELECT proposal_id FROM rule ORDER BY id DESC LIMIT 1",
                       &[],
                       |row| { prop_id = row.get(0) }
                       ) {
        Ok(_) => (),
        Err(_) => return None,
    };
    match conn.query_row("SELECT row FROM proposal ORDER WHERE id=?1",
                       &[&prop_id],
                       |row| Proposal {
                           id: row.get(0),
                           creator_id: row.get(1),
                           timestamp_proposed: row.get(2),
                           text: row.get(3),
                           target_id: row.get(4),
                           timestamp_closed: row.get(5),
                           pass_fail: row.get(6)
                       }) {
        Ok(prop) => Some(prop),
        Err(_) => None,
    }
}


pub fn current_rulemaker(conn: &rusqlite::Connection) -> Option<String> {
    let mut user_id = String::new();
    match conn.query_row("SELECT user_id FROM turn ORDER BY id DESC LIMIT 1",
                       &[],
                       |row| { user_id = row.get(0) }
                       ) {
        Ok(_) => Some(user_id),
        Err(_) => None,
    }
}
