// allow checking of proposal before posting to channel
// put the rules in
// turns
// timezones
// error-chain
// lets-encrypt
// post to wiki?
// - or at least easily query db to compare with wiki
// system for prototyping changes and new functionality!!
// - just git pull codebase, git pull db repo, hack awayz with ngrok!!!
// - oh no db not encrypted?
// players change their timezones
// roll dice
// allow download of db with username-password?
// - set db to backup automatically
// use user_ids not user_names!!!
// - get user list and register them
// - if new user, refresh user list and add new user

extern crate tiny_http;
#[macro_use]
extern crate serde_json;
extern crate url;
extern crate core;
extern crate time;
extern crate rusqlite;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate curl;

use std::collections::HashMap;
use tiny_http::{Server, Response, Header};
use core::str::FromStr;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufReader;
use std::io::BufRead;
use curl::easy::Easy;
use url::Url;
use std::io::Read;
use std::io::Write;


mod nomic_db;
mod proposal;
mod utils;

use proposal::Proposal;

const SLACK_WEB_API_URL: &str = "https://slack.com/api/";
const CREDENTIALS_FILE_PATH: &str = "/home/rowan/documents/apps/autonomic/credentials";

#[derive(Debug)]
pub struct Player {
    pub id: i32,
    pub slack_id: String,
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
    env_logger::init().unwrap();
    let now = time::get_time();
    info!("Starting up on {}, unix-time: {}.{}",
          time::at_utc(now).asctime(),
          now.sec,
          now.nsec);

    let server = Server::http("0.0.0.0:6677").unwrap();

    //dev
    let conn = rusqlite::Connection::open_in_memory().unwrap();

    // Get list of users
    let creds = read_credentials();
    let users = slack_web_call("users.list",
                               vec![("token",
                                     &creds["oauth_token"]
                                     .to_string()
                                     .replace("\"","")[..])]);
    debug!("users: {}", users);


    nomic_db::init_db(&conn);

    //dev
    nomic_db::print_player_table(&conn);
    
    // infinite loop of request handling
    for mut request in server.incoming_requests() {
        let mut body = String::new();
        request.as_reader().read_to_string(&mut body).unwrap();

        debug!("received request!\n\
               method: {:?}\n\
               url: {:?}\n\
               headers: {:?}\n\
               body: {}",
            request.method(),
            request.url(),
            request.headers(),
            body.replace('&', "\n\t"));
    
        // dev
        //let proposal = Proposal { 
        //    id: 1,
        //    creator_id: String::from("fishhead"),
        //    timestamp_proposed: time::get_time().sec,
        //    text: String::from("No things shall blank"),
        //    target_id: None,
        //    timestamp_closed: None,
        //    pass_fail: None
        //};

        let body: HashMap<String,String> =
            url::form_urlencoded::parse(&body.as_bytes())
            .into_owned()
            .collect();
        //let body: serde_json::Value = serde_json::from_str(&body[..]).unwrap();
        //debug!("Message body json: {}", body);

        let response =
            // Slash commands won't have a "payload", but message interactions
            // like button presses will
            match body.get("payload") {

                // Slash command
                None => {

                    match body.get("command") {
                        Some(slashie) => {
                            debug!("command: {}\nuser_name: {}\ntext: {}",
                                     body.get("command").unwrap(),
                                     body.get("user_name").unwrap(),
                                     body.get("text").unwrap());

                            match &slashie.to_string()[..] {
                                "/vote" => 
                                    match handle_vote_slash(&conn,
                                                            body.get("user_id").unwrap()
                                                            ) {
                                        Ok(resp) => resp,
                                        Err(e) => {
                                            error!("{}", e);
                                            continue;
                                        },
                                    },
                                "/turns" => turns_slash_response(),

                                "/proposal" => handle_proposal_slash(&conn, &body),

                                _ => String::from("I haven't implemented this command yet, soz!"),
                            }
                        },

                        // Maybe change this to return a static acknowlegment page
                        // if it's a GET request.
                        None => {
                            info!("Received a request that didn't fit any pattern: {:?}",
                                  body);
                            continue;
                        },
                    }
                },

                Some(payload) => {
                    //dev
                    debug!("payload from message ineraction: {}", payload);

                    let payload: serde_json::Value = serde_json::from_str(payload).unwrap();

                    //dev
                    debug!("actions: {}", payload["actions"]);
                    debug!("callback_id: {}", payload["callback_id"]);
                    debug!("username: {}", payload["user"]["name"]);
                    debug!("value: {}", payload["actions"][0]["value"]);


                    match &payload["callback_id"]
                        .to_string()
                        .replace('"', "")[..] {

                        "vote" => handle_vote_interaction(&payload),

                        "proposal" => handle_proposal_interaction(&payload),

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


fn handle_vote_interaction(payload: &serde_json::Value) -> String {
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


fn handle_vote_slash(conn: &rusqlite::Connection, user_id: &str)
        -> Result<String,String> {
    let proposal = match nomic_db::get_current_proposal(conn) {
        Some(prop) => prop,
        None => return Err(String::from(
            "Error: There is no current active proposal")),
    };

    let username =
        nomic_db::get_username(conn, &proposal.creator_id[..])?;
    Ok(json!({
        "text": format!("Vote on prop {} by {}", proposal.id, username),
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
    }).to_string())
}


fn handle_proposal_slash(conn: &rusqlite::Connection,
                         body: &HashMap<String,String>)
        -> String {

   // Check if it's this users turn
   let current_rulemaker =
       match nomic_db::current_rulemaker(conn) {
           Some(user_id) => user_id,
           None => {
               error!("It's no-ones turn!");
               String::from("Ooh, looks like the database isn't \
                            set up or something. Everybody freak out!")
           },
   };
   if &current_rulemaker[..] ==
           body.get("user_id").unwrap() {
       String::from("Error: it's not your \
                    turn to propose a rule")
   } else {
       match proposal::interpreter::interpret(
               &conn,
               &body.get("text")
                   .unwrap()
                   .to_string()[..],
               &body.get("user_id")
               .unwrap()
               .to_string()[..]
               ) {
           Ok(msg) =>
               json!({
                   "text": msg,
                   "attachments": [
                   { "text":
                       body.get("text")
                           .unwrap()
                           .to_string(),
                     "fallback": "You can't vote from this device, soz",
                     "attachment_type": "default",
                     "callback_id": "vote",
                     "actions": [
                         {
                             "name": "submit",
                             "text": "Submit",
                             "type": "button",
                             "value": "yes"
                         },
                         {
                             "name": "don't submit",
                             "text": "Don't Submit",
                             "type": "button",
                             "value": "no"
                         },
                     ]
                   }]
               }).to_string(),

           Err(msg) =>
               json!({
                   "text": msg,
                   "color": "#FF3333",
                   "attachments": [
                   { "text": body.get("text")
                       .unwrap()
                           .to_string() }
                   ]
               }).to_string(),
       }
   }
}


fn handle_proposal_interaction(payload: &serde_json::Value) -> String {
    // check answer to submit button
    // - 'submit'
    //   - call proposal::amend or whatever
    //   - put amend args in the original message,
    //     acess them here
    // - 'don't submit'
    //   - just print back the message for copy paste with /proposal at the start
    //
    String::from("This is just so it compiles :)")
}


fn slack_web_call(method: &str, fields: Vec<(&str,&str)>) -> serde_json::Value {
    let mut url = String::from(SLACK_WEB_API_URL);
    url = url + method + "?";
    for (key, val) in fields {
        url = url + key + "=" + val + "&";
    }
    let response = curl_req_blocking(&url[..]);
    serde_json::from_str(&response[..]).unwrap()
}


fn curl_req_blocking(req: &str) -> String {
    debug!("curl request to send: {}", req);

    let mut buf: Vec<u8> = Vec::new();
    let mut easy = Easy::new();
    easy.url(req).unwrap();
    {
        let mut transfer = easy.transfer();
        transfer.write_function(|data| {
            buf.extend_from_slice(data);
            Ok(data.len())
        }).unwrap();
        transfer.perform().unwrap();
    }
    let response = String::from_utf8(buf).unwrap();

    debug!("data from slack req: {}", response);
    return response;
}


fn read_credentials() -> serde_json::Value {
    let mut cred_file = File::open(CREDENTIALS_FILE_PATH).unwrap();
    let mut creds = String::new();
    cred_file.read_to_string(&mut creds).unwrap();
    debug!("cred file contents: {}", creds);
    let creds: serde_json::Value = serde_json::from_str(&creds[..]).unwrap();
    debug!("cred file json parse: {}", creds);
    debug!("Reading creds:\n\
            verification token: {}\n\
            client ID: {}\n\
            client secret exists: {}\n\
            oauth_token: {}",
            creds["verification_token"],
            creds["client_id"],
            creds["client_secret"] != serde_json::Value::Null,
            creds["oauth_token"]);
    return creds;
}
