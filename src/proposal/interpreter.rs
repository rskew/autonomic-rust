use rusqlite;
use std::fmt;

use nomic_db;
use utils::remove_punctuation;

use proposal;
use proposal::Proposal;


//const ARTICLES: [&str; 8] = ["to", "the",
//                             "rule", "of",
//                             "too", "for",
//                             "at","off"];
//const TRANSMUTE_WORDS: [&str; 12] = ["to", "become",
//                                     "make", "it",
//                                     "making", "changing",
//                                     "transmuting", "into",
//                                     "becoming", "too",
//                                     "for", "at"];

const TYPE_BUT_NO_TARGET_MSG: &str =
    "Error: no target id found.\n\
    If you're going to change another rule, \
    you need to specify which one!\n\
    Please put the id of the target rule \
    right after the proposal type, like this:\n\
    'Rule 999, amendment to rule 998: \
    Give me all the treasure rather than sharing it equally' or something :)";

const PROP_TYPES: &str = ":small_blue_diamond: Enactment\n\
                          :small_blue_diamond: Amendment\n\
                          :small_blue_diamond: Transmutation\n\
                          :small_blue_diamond: Repeal";


#[derive(PartialEq)]
#[derive(Clone)]
pub enum Mutability {
    Mutable,
    Immutable,
}


impl Mutability {
    pub fn from_str(s: &str) -> Result<Mutability,String> {
        match &s.to_lowercase()[..] {
            "mutable" => Ok(Mutability::Mutable),
            "immutable" => Ok(Mutability::Immutable),
            _ => Err(format!(
                    "Error: {} isn't one of the two valid mutability types: \
                    Mutable and Immutable",
                    s)),
        }
    }

    pub fn not(&self) -> Mutability {
        match self {
            &Mutability::Mutable => Mutability::Immutable,
            &Mutability::Immutable => Mutability::Mutable,
        }
    }
}

impl fmt::Display for Mutability {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Mutability::Mutable => write!(f, "Mutable"),
            &Mutability::Immutable => write!(f, "Immutable"),
        }
    }
}


#[derive(PartialEq)]
#[derive(Clone)]
enum Token {
    Enact,
    Repeal,
    Amend,
    Transmute,
    Id(i32),
    Muta(Mutability),
    Text
}


enum ParsedCommand {
    Enactment(i32, String),
    Amendment(i32, String, i32),
    Repealment(i32, String, i32),
    Transmutation(i32, String, i32, Mutability),
    NoKeyWords,
    NotIdFirst,
    IdOnly,
    IdNoType,
    NoTarget,
    NoTargetAfterType,
    NoTransmuteDir,
    NoTransmuteDirAfterTarget,
    Invalid
}



// Match keywords
//
// Could extend this with approximate string matching
fn tokenize_lexeme(word: &str) -> Token {
    match &remove_punctuation(&word.to_lowercase()[..])[..] {
        "enactment" | "enact" |
        "enactify"  | "enacterate" => Token::Enact,
        "amendment" | "amend" |
        "amendify" => Token::Amend,
        "repeal" | "repealify" => Token::Repeal,
        "transmutation" | "transmute" |
        "transmutate" | "transmuterate" => Token::Transmute,
        other => {
            if let Ok(num) = other.parse::<i32>() {
                Token::Id(num)
            } else if let Ok(muta) = Mutability::from_str(other) {
                Token::Muta(muta)
            } else {
                Token::Text
            }
        }
    }
}


fn interesting_token(token: &Token) -> bool {
    match token {
        &Token::Text => false,
        _ => true
    }
}


// First converts each word to keyword Token, prunes off
// all non-keyword Tokens then uses 'parse' to implicitly
// matches the sequence of keywords to valid and invalid commands.
pub fn interpret(conn: &rusqlite::Connection,
                 prop: &str,
                 creator_id: &str)
        -> Result<String,String> {
    //let mut tokens = prop.split_whitespace()
    //    .map(|word| tokenize_lexeme(word))
    //    .filter(interesting_token);

    let mut tokens: Vec<Token> = prop.split_whitespace()
        .map(|word| tokenize_lexeme(word))
        .filter(interesting_token)
        .collect();

    match parse(tokens, creator_id) {

        ParsedCommand::Enactment(prop_id, creator_id) => {
            check_prop_id(conn, prop_id)?;
            Ok(String::from("Cool rule, double check it and then submit!"))
            //proposal::enact(conn, prop, prop_id, &creator_id[..])
        },
                    
        ParsedCommand::Amendment(prop_id, creator_id, target_id) => {
            check_prop_id(conn, prop_id)?;
            check_target_prop_id(conn, target_id)?;
            proposal::amend(conn, prop, prop_id, &creator_id[..], target_id)
        },


        ParsedCommand::Repealment(prop_id, creator_id, target_id) => {
            check_prop_id(conn, prop_id)?;
            check_target_prop_id(conn, target_id)?;
            proposal::repeal(conn, prop, prop_id, &creator_id[..], target_id)
        },

        ParsedCommand::Transmutation(prop_id, creator_id, target_id, muta) => {
            check_prop_id(conn, prop_id)?;
            check_target_prop_id(conn, target_id)?;
            check_mutability_of_target(conn, target_id, muta)?;
            proposal::repeal(conn, prop, prop_id, &creator_id[..], target_id)
        },

        ParsedCommand::NoKeyWords => Err(String::from(
                    "Error: Please add some words to your proposal!")),

        ParsedCommand::NotIdFirst => Err(String::from(
                    "Error: Please start your proposal by declaring \
                    its identification number, cheers!")),

        ParsedCommand::IdOnly => Err(format!(
                    "Error: Please include the proposal type :)\n\
                    Valid types are:\n{}", PROP_TYPES)),

        ParsedCommand::IdNoType => Err(format!(
                    "Error: no proposal type found.\n\
                    Please begin your proposal with it's type. For example:\n\
                    'Rule 999, Enactment: \
                    Nomicball! Nomic is now also a ball game.'\n\
                    Valid types are:\n{}", PROP_TYPES)),

        ParsedCommand::NoTarget =>
                    Err(String::from(TYPE_BUT_NO_TARGET_MSG)),

        ParsedCommand::NoTargetAfterType =>
                    Err(String::from(TYPE_BUT_NO_TARGET_MSG)),

        ParsedCommand::NoTransmuteDir => Err(String::from(
                    "Error: Please provide a direction of transmutation. \
                    I know it can only go one way, \
                    but Autonomic would like to \
                    encourage clarity for human's sake :)")),

        ParsedCommand::NoTransmuteDirAfterTarget => Err(String::from(
                    "Error: Please specify the direction of transmutation :)"
                    )),
                                
        ParsedCommand::Invalid => Err(String::from(
                    "Invalid parsing of the proposal, woops!"
                    )),
    }
}

//    // Get the id of the proposal
//    let proposal_id_token = 
//        token_please(tokens.next(), "Error: Please add some words to your proposal!")?;
//    let proposal_id = match proposal_id_token {
//        Token::Id(num) => num,
//        _ => return Err(String::from(
//                "Error: Please start your proposal by declaring \
//                its identification number, cheers!"
//                )),
//    };
//
//    // Check if the id is the next in the sequence
//    let next_prop_id =
//        match nomic_db::last_proposal_id(conn) {
//            Some(num) => num + 1,
//            None => 300,
//        };
//    if proposal_id != next_prop_id {
//        return Err(String::from(
//                &format!("Error: that rule id number isn't the \
//                next one in the sequence, try again!\n\
//                The next rule number should be {}", next_prop_id)[..]
//                ));
//    }
//
//    // Parse the type of proposal
//    let prop_type =  token_please(tokens.next(),
//            &format!("Error: Please include the proposal type :)\n\
//                    Valid types are:\n{}", PROP_TYPES)[..])?;
//    match prop_type {
//        // Simply adding a new rule
//        Token::Enact => proposal::enact(conn, prop, proposal_id, creator_id),
//
//        // Altering another rule
//        Token::Amend | Token::Repeal | Token::Transmute => {
//
//            // Get the target rule to be altered
//            let target_id_token =
//                token_please(tokens.next(), TYPE_BUT_NO_TARGET_MSG)?;
//            let target_id = match target_id_token {
//                Token::Id(num) => num,
//                _ => return Err(String::from(
//                        TYPE_BUT_NO_TARGET_MSG)),
//            };
//
//            // Check if target_is is a current rule
//            if !nomic_db::is_current_rule(conn, target_id) {
//                return Err(String::from(
//                        &format!(
//                        "Error: The target id {} is not a current rule!"
//                        , target_id)[..]));
//            }
//
//
//            // Hone in on the type
//            match prop_type {
//
//                Token::Amend => proposal::amend(conn, prop, proposal_id,
//                                      creator_id, target_id),
//
//                Token::Repeal => proposal::repeal(conn, prop, proposal_id,
//                                        creator_id, target_id),
//
//                Token::Transmute => {
//
//                    // Get direction to transmute
//                    let trans_dir_token =
//                        token_please(tokens.next(),
//                        NO_TRANSMUTE_DIRECTION_MSG)?;
//                    let trans_dir = match trans_dir_token {
//                        Token::Muta(muta) => muta,
//                        _ => return Err(String::from(
//                                "Error: Please specify the direction of transmutation :)"
//                                )),
//                    };
//
//                    // Check direction is correct
//                    let target_mutability =
//                        nomic_db::check_prop_mutability(conn, target_id)?;
//                    if trans_dir != target_mutability.not() {
//                        return Err(format!(
//                            "Error: rule {} can't be transmuted to {}",
//                            target_id,
//                            trans_dir));
//                    };
//
//                    proposal::transmute(conn, prop, proposal_id,
//                                        creator_id, target_id,
//                                        trans_dir)
//                },
//
//                _ => return Err(String::from(
//                        "Invalid parsing of the proposal, woops!"
//                        )),
//            }
//        },
//
//        _ => return Err(String::from(format!(
//                    "Error: no proposal type found.\n\
//                    Please begin your proposal with it's type. For example:\n\
//                    'Rule 999, Enactment: \
//                    Nomicball! Nomic is now also a ball game.'\n\
//                    Valid types are:\n{}", PROP_TYPES))),
//    }
//}


// State machine style parser for proposal slash command.
// Matches the sequence of keywords to valid and invalid commands.
fn parse(tokens: Vec<Token>, creator_id: &str) -> ParsedCommand {
    let mut tokens = tokens.into_iter();

    // Get the id of the proposal
    let proposal_id_token = match tokens.next() {
        Some(token) => token.clone(),
        None => return ParsedCommand::NoKeyWords,
    };
    let proposal_id = match proposal_id_token {
        Token::Id(num) => num,
        _ => return ParsedCommand::NotIdFirst
    };

    // Parse the type of proposal
    let prop_type = match tokens.next() {
        Some(token) => token,
        None => return ParsedCommand::IdOnly,
    };

    match prop_type {
        // Simply adding a new rule
        Token::Enact =>
            ParsedCommand::Enactment(proposal_id,
                                     String::from(creator_id)),

        // Altering another rule
        Token::Amend | Token::Repeal | Token::Transmute => {

            // Get the target rule to be altered
            let target_id_token = match tokens.next() {
                Some(token) => token,
                None => return ParsedCommand::NoTarget,
            };
            let target_id = match target_id_token {
                Token::Id(num) => num,
                _ => return ParsedCommand::NoTargetAfterType,
            };

            // Hone in on the type
            match prop_type {

                Token::Amend =>
                    ParsedCommand::Amendment(proposal_id,
                                             String::from(creator_id),
                                             target_id),

                Token::Repeal =>
                    ParsedCommand::Repealment(proposal_id,
                                              String::from(creator_id),
                                              target_id),

                Token::Transmute => {

                    // Get direction to transmute
                    let trans_dir_token = match tokens.next() {
                        Some(token) => token,
                        None => return ParsedCommand::NoTransmuteDir,
                    };
                    let trans_dir = match trans_dir_token {
                        Token::Muta(muta) => muta,
                        _ => return ParsedCommand::NoTransmuteDirAfterTarget,
                    };

                    ParsedCommand::Transmutation(proposal_id,
                                                 String::from(creator_id),
                                                 target_id,
                                                 trans_dir)
                },

                _ => return ParsedCommand::Invalid,
            }
        },

        _ => return ParsedCommand::IdNoType,
    }
}
//
//
//fn token_please(maybe_token: Option<Token>, err_msg: &str)
//    -> Result<Token,String> {
//        match maybe_token {
//            Some(token) => Ok(token),
//            None => Err(String::from(err_msg)),
//        }
//}

fn check_prop_id(conn: &rusqlite::Connection, prop_id: i32)
        -> Result<(),String> {
    // Check if the id is the next in the sequence
    let next_prop_id =
        match nomic_db::last_proposal_id(conn) {
            Some(num) => num + 1,
            None => 300,
        };
    if prop_id != next_prop_id {
        Err(format!(
            "Error: that rule id number isn't the \
            next one in the sequence, try again!\n\
            The next rule number should be {}"
            , next_prop_id))
    } else {
        Ok(())
    }
}


fn check_target_prop_id(conn: &rusqlite::Connection, target_id: i32)
        -> Result<(),String> {
    // Check if target_id is a current rule
    if !nomic_db::is_current_rule(conn, target_id) {
        return Err(String::from(
                &format!(
                "Error: The target id {} is not a current rule!"
                , target_id)[..]));
    };
    Ok(())
}


fn check_mutability_of_target(conn: &rusqlite::Connection,
                              target_id: i32,
                              trans_dir: Mutability)
        -> Result<(),String> {
    let target_mutability =
        nomic_db::check_prop_mutability(conn, target_id)?;
    if trans_dir != target_mutability.not() {
        return Err(format!(
            "Error: rule {} can't be transmuted to {}",
            target_id,
            trans_dir));
    };
    Ok(())
}
