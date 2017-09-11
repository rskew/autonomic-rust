// proposal_json
// allow checking of proposal before posting to channel
// turns
// timezones
// sqlite
// error-chain
// lets-encrypt
// post to wiki?
// - or at least easily query db to compare with wiki
// system for prototyping changes and new functionality!!
// - just git pull codebase, git pull db repo, hack awayz with ngrok!!!
// players change their timezones
// roll dice
// allow download of db with username-password?
// - set db to backup automatically
// use user_ids not user_names!!!

extern crate tiny_http;
#[macro_use]
extern crate serde_json;
extern crate url;
extern crate core;
extern crate time;
extern crate rusqlite;
#[macro_use]
extern crate log;

use std::collections::HashMap;
use tiny_http::{Server, Response, Header};
use core::str::FromStr;

mod nomic_db;
mod proposal;
mod utils;

use proposal::Proposal;
use utils::remove_punctuation;

pub const BULLET_POINT: &str = ":small_blue_diamond:";

#[derive(Debug)]
pub struct Player {
    pub id: i32,
    pub username: String,
    pub points: i32,
    pub timezone: String
}

#[derive(Debug)]
pub struct Vote {
    pub id: i32,
    pub proposal_id: i32,
    pub user_id: i32,
    pub timestamp: i64,
    pub yes_no: bool
}

#[derive(Debug)]
pub struct Rule {
    pub id: i32,
    pub proposal_id: i32,
    pub mutable: bool,
    pub amendment_id: Option<i32>
}


fn main() {
    println!("Does this work: {}", remove_punctuation(String::from("things, things; thing! thing? thing. goof\"")));

    let server = Server::http("0.0.0.0:6677").unwrap();

    //dev
    let conn = rusqlite::Connection::open_in_memory().unwrap();

    nomic_db::init_db(&conn);

    //dev
    nomic_db::print_player_table(&conn);
    
    for mut request in server.incoming_requests() {
        let mut body = String::new();
        request.as_reader().read_to_string(&mut body).unwrap();

        // dev
        println!("received request!\nmethod: {:?}\nurl: {:?}\nheaders: {:?}\nbody: {}",
            request.method(),
            request.url(),
            request.headers(),
            body.replace('&', "\n\t")
            // grab value associated with "text"
        );
    
        // dev
        let proposal = Proposal { 
            id: 1,
            creator_id: 1,
            username: nomic_db::get_username_from_id(&conn, 1).unwrap(),
            timestamp_proposed: 0,
            text: String::from("No things shall blank"),
            timestamp_closed: None,
            pass_fail: None
        };


        let body: HashMap<String,String> =
            url::form_urlencoded::parse(&body.as_bytes()).into_owned().collect();

        let response =
            // Slash commands won't have a "payload", but message interactions
            // like button presses will
            match body.get("payload") {
                // Slash command
                None => {
                    //dev
                    if body.get("command").is_some() {
                        println!("command: {}\nuser_name: {}\ntext: {}",
                                 body.get("command").unwrap(),
                                 body.get("user_name").unwrap(),
                                 body.get("text").unwrap());
                    };

                    match body.get("command") {
                        Some(slashie) =>
                            match &slashie[..] {
                                "/vote" => vote_slash_response(&proposal),
                                "/turns" => turns_slash_response(),
                                "/proposal" => match proposal::parse_prop(&conn,
                                                                          body.get("text").unwrap(),
                                                                          //body.get("user_id").unwrap()) {
                                                                          1) {
                                    Ok(prop) => prop.to_string(),
                                    Err(msg) => msg,
                                },
                                _ => String::from("I haven't implemented this command yet, soz!"),
                            },
                        // Some other http message, do nothing.
                        // Maybe change this to return a static acknowlegment page
                        // if it's a GET request.
                        None => continue,
                    }
                },

                Some(payload) => {
                    //dev
                    println!("it was payload");
                    println!("{}", payload);

                    let payload: serde_json::Value = serde_json::from_str(payload).unwrap();

                    //dev
                    println!("actions: {}", payload["actions"]);
                    println!("callback_id: {}", payload["callback_id"]);
                    println!("username: {}", payload["user"]["name"]);
                    println!("value: {}", payload["actions"][0]["value"]);


                    match &payload["callback_id"]
                        .to_string()
                        .replace('"', "")[..] {

                        "vote" => handle_vote_interaction(payload),

                        _ => {
                            String::from("I don't know how to handle this interaction yet, soz!")
                        }
                    }
                }
            };

        let mut response = Response::from_string(response);
        response.add_header(
            Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap()
            );

        request.respond(response).unwrap();
    }
}


fn handle_vote_interaction(payload: serde_json::Value) -> String {
    // check:
    // - proposal is still valid
    // - user hasn't already voted on this proposal

    // dev
    let original_message_sec =
        i64::from_str(&payload["message_ts"].to_string()
                      .replace('"',"")
                      .split('.').next().unwrap())
        .unwrap();
    match time::get_time().sec - original_message_sec < 10 {
            true => String::from("Great vote!"),
            false => String::from("Oh no, to late!"),
        }
}


fn turns_slash_response() -> String {
    json!({
        "text": "It's someone's turn I guess?"
    }).to_string()
}


fn vote_slash_response(proposal: &Proposal) -> String {
    json!({
        "text": format!("Vote on prop {} by {}", proposal.id, proposal.username),
        "attachments": [
            {
                "text": proposal.text,
                "fallback": "You can't vote from this device, soz",
                "callback_id": "vote",
                "attachment_type": "default",
                "actions": [
                    {
                        "name": "yes",
                        "text": "Yes",
                        "type": "button",
                        "value": "yes"
                    },
                    {
                        "name": "no",
                        "text": "No",
                        "type": "button",
                        "value": "no"
                    },
                ]
            }
        ]
    }).to_string()
}


fn enact_rule(prop: &str) -> String {
    String::from("You want to enact a rule I guess!")
}

fn amend_rule(prop: &str) -> String {
    String::from("You want to amend a rule I guess!")
}

fn transmute_rule(prop: &str) -> String {
    String::from("You want to transmute a rule I guess!")
}

fn repeal_rule(prop: &str) -> String {
    String::from("You want to repeal a rule I guess!")
}
