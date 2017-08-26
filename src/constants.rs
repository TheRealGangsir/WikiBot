/// A file that holds all the constants that my bot uses.

extern crate serenity;
extern crate json;

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use self::serenity::model::GuildId;
use std::io::prelude::*;
use self::json::JsonValue;
use std::fs::{File, OpenOptions};

lazy_static! {
    pub static ref PREFIXES: Arc<Mutex<HashMap<GuildId, String>>> = Arc::new(Mutex::new(HashMap::new()));
}

pub static BOT_NAME: &'static str = "WikiBot#3868";

/// Gets the current set prefix, given an id from a guild. {{{1
pub fn get_prefix_for_guild(id: &GuildId) -> String {
    let prefixes = PREFIXES.lock().unwrap();
    if let Some(pref) = prefixes.get(&id) {
        pref.clone()
    } else {
        String::new()
    }
}

// Tests {{{1
#[cfg(test)]
mod tests {
    use self::serenity::model::GuildId;
    use super::*;

    #[test]
    fn guild_id_has_no_prefix() {
        // Dummy ID to try to retrieve from PREFIXES
        let id = GuildId::from(000000000000000000);
        let prefix = get_prefix_for_guild(&id);
        // Fail test if it returns a valid prefix
        assert_eq!(prefix, "");
    }

    #[test]
    fn adding_a_prefix_to_a_guild_id() {
        //Setup vars
        let id = GuildId::from(111111111111111111);
        {
            let mut prefixes = PREFIXES.lock().unwrap();
            let _ = prefixes.insert(id, String::from("+"));
        }

        let prefix = get_prefix_for_guild(&id);

        assert_eq!(prefix, "+");
    }
}

/// Function that installs all saved prefixes into PREFIXES from {{{1
/// a prefixes json file. Should be called on startup.
pub fn install_prefixes() {

    if let Ok(mut handle) = File::open("prefixes.json") {
        let mut json = String::new();
        let _ = handle.read_to_string(&mut json);
        let parsed_json = json::parse(json.as_str()).expect("Could not parse json read from prefixes file.");

        let mut prefixes = PREFIXES.lock().unwrap();
        for entry in parsed_json.entries() {
            let (key, value) = entry;
            let key_parsed: u64 = key.parse::<u64>().expect("Could not parse valid ID from json.");
            let _ = prefixes.insert(GuildId::from(key_parsed), format!("{}", value));
        }
        println!("Finished reading prefixes into memory.");
    } else {
        println!("Did not find file for list of prefixes, creating...");
        let mut file_handle = File::create("prefixes.json").expect("Could not create prefixes file.");
        file_handle.write_all(b"{}").expect(
            "Got error writing to newly created json file.",
            ); //Write empty json object to it
    }
}

/// Backs up prefixes to a file, reading from PREFIXES {{{1
pub fn backup_prefixes() {
    let prefixes = PREFIXES.lock().unwrap();
    let mut json = JsonValue::new_object();
    if let Ok(mut file_handle) = OpenOptions::new().write(true).truncate(true).open("prefixes.json") {
        for (key, value) in prefixes.iter() {
            json[key.0.to_string()] = value.clone().into();
        }
        json.write(&mut file_handle);
    } else {
        println!("Couldn't backup prefix list to file.")
    }
}
