use rusqlite;
use time;

use nomic_db;

pub mod interpreter;

#[derive(Debug)]
pub struct Proposal {
    pub id: i32,
    pub creator_id: String,
    pub timestamp_proposed: i64,
    pub text: String,
    pub target_id: Option<i32>,
    pub timestamp_closed: Option<time::Timespec>,
    pub pass_fail: Option<bool>
}


impl ToString for Proposal {
    fn to_string(&self) -> String {
        format!("{:?}", &self)
    }
}


fn enact(conn: &rusqlite::Connection,
         prop: &str,
         proposal_id: i32,
         creator_id: &str)
        -> Result<String,String> {
    let proposal = Proposal {
        id: proposal_id,
        creator_id: String::from(creator_id),
        timestamp_proposed: time::get_time().sec,
        text: String::from(prop),
        target_id: None,
        timestamp_closed: None,
        pass_fail: None
    };
    match nomic_db::register_proposal(conn, &proposal) {
        Ok(s) => Ok(s),
        Err(e) => {
            error!("Error enacting proposal: {}", e);
            Err(String::from("Error enacting the proposal, sorry!"))
        },
    }
}


fn amend(conn: &rusqlite::Connection,
         prop: &str,
         proposal_id: i32,
         creator_id: &str,
         target_id: i32) -> Result<String,String> {
    // add amendment id to rule being amended
    return Err(String::from("Do an amendment!"))
}


fn repeal(conn: &rusqlite::Connection,
          prop: &str,
          proposal_id: i32,
          creator_id: &str,
          target_id: i32) -> Result<String,String> {
    return Err(String::from("Repeal it then re-squeal it!?"))
}


fn transmute(conn: &rusqlite::Connection,
             prop: &str,
             proposal_id: i32,
             creator_id: &str,
             target_id: i32,
             transmute_direction: interpreter::Mutability)
        -> Result<String,String> {
    return Err(String::from("transmuting yeah!"))
}
