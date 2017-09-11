use rusqlite;

use std::iter::Map;

use nomic_db;
use utils::remove_punctuation;

const ARTICLES: [&str; 8] = ["to", "the", "rule", "of", "too", "for", "at", "off"];
const TYPE_BUT_NO_TARGET_MSG: &str = "If you're going to change another rule, you need to specify which one!\n\
                                 Please put the id of the target rule right after the proposal type, like this:\n\
                                 'Rule 999, amendment to rule 998: Give me all the treasure rather than sharing it equally' or something :)";

fn prop_types() -> String {
    String::from("bullet_point Enactment\n\
                 bullet_point Amendment\n\
                 bullet_point Transmutation\n\
                 bullet_point Repeal")
                 .replace("bullet_point", ::BULLET_POINT)
}

#[derive(Debug)]
pub struct Proposal {
    pub id: i32,
    pub creator_id: i32,
    pub username: String,
    pub timestamp_proposed: i64,
    pub text: String,
    pub target_id: Option<i32>,
    pub timestamp_closed: Option<i64>,
    pub pass_fail: Option<bool>
}


impl ToString for Proposal {
    fn to_string(&self) -> String {
        format!("{:?}", &self)
    }
}


pub fn parse_prop(conn: &rusqlite::Connection, prop: &str, creator_id: i32) -> Result<Proposal,String> {
    let mut words = prop.split_whitespace().map(|word| remove_punctuation(word.to_lowercase()));
    let mut first_word =
            next_word(words.next(), String::from("Please add some words to your proposal!"))?;

    // Strip the leading 'Rule' or such word
    if first_word == "rule" {
        first_word =
            next_word(words.next(), String::from("Please include some more words in your proposal :)"))?;
    };

    // Get the identification number of the proposal
    let proposal_id: i32 = match first_word.parse() {
            Ok(num) => num,
            Err(_) => return Err(String::from(
                    "Please start your proposal by declaring its identification number, cheers!"
                    )),
    };
    // Check proposal id for uniqueness
    if nomic_db::id_exists(&conn, "proposal", proposal_id) {
        return Err(String::from("There is already a proposal with this number!"));
    };

    // Check if the id is the next in the sequence
    if proposal_id != nomic_db::next_proposal_id(conn) {
        return Err(String::from(
                "Hmm, that rule id number isn't the next one in the sequence, try again!"
                ));
    }

    // Parse the type of proposal
    let prop_type = next_word(words.next(), String::from(
            format!("Please include more stuff than just the number in your proposal :)\
                    Valid types are:\n{}", prop_types())
            ));
    match &prop_type[..] {

        "enactment" | "enact" => enact(prop, proposal_id, creator_id),

        "amendment" | "amend" | "transmutation" | "transmute" | "repeal" => {
            let mut target = next_word(words.next(), String::from(TYPE_BUT_NO_TARGET_MSG))?;

            // Trim the link-words
            while ARTICLES.contains(&&target[..]) {
                target = next_word(words.next(), String::from(TYPE_BUT_NO_TARGET_MSG))?;
            }

            // Get the id of the target rule that will be altered
            let target_id: i32 = match target.parse() {
                    Ok(num) => num,
                    Err(_) => return Err(String::from(TYPE_BUT_NO_TARGET_MSG)),
            };

            // Hone in on the type
            match &prop_type[..] {
                "amendment" | "amend" => amend(prop, proposal_id, creator_id, target_id),
                "transmutation" | "transmute" => return Err(String::from("transmuting yeah!")),
                "repeal" => return Err(String::from("Repeal it then re-squeal it!?")),
                _ => return Err(String::from("Oh crap, shouldn't be here in the code woops!")),
            }
        },
        _ => return Err(String::from(format!("Please begin your proposal with it's type. Valid types are:\n{}", prop_types()))),
    }
}


fn next_word(maybe_word: Option<String>, err_msg: String) -> Result<String,String> {
    match maybe_word {
        Some(word) => Ok(word),
        None => Err(err_msg),
    }
}


fn enact(prop: &str, proposal_id: i32, creator_id: i32) -> Result<Proposal,String> {
//    Proposal {
//        id: proposal_id,
//        creator_id: 

    return Err(String::from("Do an enactify!"))
}


fn amend(prop: &str, proposal_id: i32, creator)id: i32, target_id: i32) -> Result<Proposal,String> {
    // add amendment id to rule being amended
    return Err(String::from("Do an amendment!"))
}

//fn proposal_slash_response(body: &HashMap<String,String>) -> String {
//    // if amendment or repeal, check if target is mutable.
//    // if transmutation, check the direction is correct?
//    match body.get("text") {
//        Some(prop) => {
//            String::from("things")
//        },
//
//        None => String::from("Please type your proposal after the slash command '/proposal', cheers!"),
//    }
//}
