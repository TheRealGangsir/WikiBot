use json;
use json::JsonValue;

use reqwest;

use serenity::model::Message;
use serenity::utils::Colour;

use std::io::Read;

use common_funcs::*;
use constants::*;
use levenshtein::*;

/// Struct used to hold a bunch of data about a mod, for easy passing {{{1
#[derive(Debug)]
struct Mod {
    pub creation_date: String,
    pub last_updated: String,
    pub thumb: String, //The thumb image of the mod
    pub name: String, //Internal name of the mod
    pub author: String,
    pub link: String,
    pub download_count: u64,
    pub latest_version: String,
    pub factorio_version: String, //Latest factorio version the mod supports
    pub source_path: String,
    pub homepage: String,
    pub dependencies: String,
    pub summary: String,
    pub title: String, //Pretty title of the mod
    pub tag: Option<String>, //What tag the mod has
}

/// Creates an embed based on the recieved mod data. {{{1
/// Returns true if successful.
fn make_mod_embed(modification: Mod, message: &Message) -> bool {
    let tag_str = modification.tag.as_ref().map(String::as_str).unwrap_or(
        "Not tagged.",
    );

    let result = message.channel_id.send_message(|a| {
        a.embed(|b| {
            b.description(&modification.summary)
                .author(|c| c.name(&modification.title).url(&modification.link))
                .thumbnail(&modification.thumb)
                .color(Colour::from_rgb(255, 34, 108))
                .timestamp(message.timestamp.to_rfc3339())
                .field(|c| c.name("Author").value(&modification.author))
                .field(|c| {
                    c.name("Downloads").value(&format!(
                        "{} downloads",
                        modification.download_count
                    ))
                })
                .field(|c| c.name("Source code").value(&modification.source_path))
                .field(|c| c.name("Homepage").value(&modification.homepage))
                .field(|c| c.name("Last updated").value(&modification.last_updated))
                .field(|c| c.name("Dependencies").value(&modification.dependencies))
                .field(|c| c.name("Tagged").value(&tag_str))
                .field(|c| c.name("Created on").value(&modification.creation_date))
                .field(|c| {
                    c.name("Latest mod version").value(
                        &modification.latest_version,
                    )
                })
                .field(|c| {
                    c.name("Factorio version").value(
                        &modification.factorio_version,
                    )
                })
        })
    });
    if let Err(error) = result {
        println!("Got error with sending embed of mod: {:?}", error);
        return false;
    }
    true
}

/// Turns a jsonValue into a Mod. Assumes this is a direct mod json entry. {{{1
/// This can be provided either by direct link, or by giving it one of the results
/// returned by a mod api search.
fn parse_json_into_mod(json: &JsonValue) -> Mod {
    let (_, downloads, _) = json["downloads_count"].as_number().unwrap().as_parts();
    let mut thumbnail = String::from("http://i.imgur.com/ckaei9P.png");
    let mut homepage = String::from("None");
    let mut source = String::from("None");
    if !json["first_media_file"].is_null() {
        thumbnail = format!("{}", json["first_media_file"]["urls"]["thumb"]);
    }

    // Formatting for home page/source page
    if !json["homepage"].is_null() && json["homepage"] != "" {
        homepage = format!("[Home]({})", json["homepage"]);
    }

    if !json["github_path"].is_null() && json["github_path"] != "" {
        source = format!("[Source](https://github.com/{})", json["github_path"]);
    }

    // Format tag correctly for untagged mods
    let mut tag = None;
    if !json["tags"].is_empty() && !json["tags"].is_null() {
        tag = Some(format!("{}", json["tags"][0]["title"]))
    }

    // Fix dates
    let mut date = format!("{}", json["created_at"]);
    if let Some(index) = date.find(" ") {
        date.truncate(index as usize);
    }

    let mut update_date = format!("{}", json["updated_at"]);
    if let Some(index) = update_date.find(" ") {
        update_date.truncate(index as usize);
    }

    let mut deps = String::from("No dependencies.");
    let dependencies_json = &json["latest_release"]["info_json"]["dependencies"];
    if !dependencies_json.is_null() && !dependencies_json.is_empty() {
        deps.clear();
        let mut deps_counter = 0;
        for entry in dependencies_json.members() {
            let entry_str = format!("{}\n", entry);
            // If the dependency is optional, just count it
            if entry_str.starts_with("?") {
                deps_counter += 1;
            } else {
                deps.push_str(&entry_str);
            }
        }
        if deps_counter == 1 {
            deps.push_str(&format!("...and {} optional dependency.", deps_counter));
        } else if deps_counter > 1 {
            deps.push_str(&format!("...and {} optional dependencies.", deps_counter));
        }
    }

    // Make and return the mod
    Mod {
        creation_date: date,
        last_updated: update_date,
        name: format!("{}", json["name"]),
        author: format!("{}", json["owner"]),
        summary: format!("{}", json["summary"]),
        title: format!("{}", json["title"]),
        latest_version: format!("{}", json["latest_release"]["version"]),
        factorio_version: format!("{}", json["latest_release"]["factorio_version"]),
        tag: tag,
        thumb: thumbnail,
        download_count: downloads,
        homepage: homepage,
        dependencies: deps,
        source_path: source,
        link: format!(
            "https://mods.factorio.com/mods/{}/{}",
            json["owner"],
            json["name"]
        ).replace(" ", "%20"),
    }
}


/// Makes a request, either returning empty, or the successful mod json {{{1
fn make_request(request: &String) -> JsonValue {
    let response = reqwest::get(
        format!("https://mods.factorio.com/api/mods?q={}", request).as_str(),
    );

    if let Ok(mut response) = response {
        if response.status().is_success() {
            let mut json = String::new();
            let _ = response.read_to_string(&mut json);
            return json::parse(&json).unwrap_or(JsonValue::new_object());
        } else {
            return JsonValue::new_object();
        }
    } else {
        return JsonValue::new_object();
    }
}

/// Makes an embed of search results. Takes a json array, and returns true {{{1
/// if able to make an embed of all the results

fn make_search_results_embed(message: &Message, results: JsonValue) -> bool {
    message
        .channel_id
        .send_message(|a| {
            a.embed(|b| {
                b.title("Search results:")
                    .description(&serialize_search_results(&results))
                    .color(Colour::from_rgb(255, 34, 108))
                    .timestamp(message.timestamp.to_rfc3339())
            })
        })
        .is_ok()
}

/// Takes a json array as input, returns a string of the search results serialized. {{{1
fn serialize_search_results(results: &JsonValue) -> String {
    let mut final_string = String::new();

    for entry in results.members().take(10) {
        let version = format!("{}", entry["latest_release"]["factorio_version"]);
        let version: f32 = version.parse::<f32>().unwrap_or(0.0);
        if version < 0.15 {
            continue; //Filter mods that aren't for 0.15 or higher
        }

        let mut encoded_name = format!("{}", entry["name"]);
        // URLS can't have spaces
        encoded_name = encoded_name.replace(" ", "%20");
        final_string += format!(
            "[{}](https://mods.factorio.com/mods/{}/{}) by {}\n",
            entry["title"],
            entry["owner"],
            encoded_name,
            entry["owner"]
        ).as_str();
    }
    if final_string.is_empty() {
        return String::from("No results for current factorio stable version or higher.");
    }
    final_string
}

/// Links a mod into chat based on args. {{{1
command!(linkmod(_context, message) {
    let request = fix_message(message.content_safe(), "linkmod", &get_prefix_for_guild(&message.guild_id().unwrap()));
    let _ = message.channel_id.broadcast_typing();

    // Check arg validity
    if request.is_empty() {
        if let Err(_) = send_error_embed(&message, "Expected a mod to search for.") {
            say_into_chat(&message, "Expected a mod to search for.");
        }
        return Err(String::from("User didn't provide an argument."));
    }

    // Make the mod api request
    let returned = make_request(&request);
    if !returned.is_empty() {
        let returned_results = &returned["results"];

        if !returned_results.is_null() && !returned_results.is_empty() {
            // If there's only one search result
            if returned_results.len() == 1 {
                let modification = parse_json_into_mod(&returned_results[0]);
                if !make_mod_embed(modification, &message) {
                    say_into_chat(&message, "Unable to make an embed here.");
                    return Err(String::from("Couldn't make an embed."));
                } else {
                    return Ok(());
                }
            }
            // More than one search result, so let's list them
            for entry in returned_results.members() {
                let modification = parse_json_into_mod(&entry);

                // Check if the match is close enough, both on the
                // internal name and the title
                if levenshtein(&modification.name, &request) <= 3
                    || levenshtein(&modification.title, &request) <= 3 {
                        // Got a match on this entry, so let's send it
                        if !make_mod_embed(modification, &message) {
                            say_into_chat(&message, "Unable to make an embed here.");
                            return Err(String::from("Couldn't make an embed."));
                        } else {
                            return Ok(());
                        }
                    }
            }
            // At this point, it hasn't found an exact match,
            // so let's just make an embed with all the results it found
            if !make_search_results_embed(&message, returned_results.clone()) {

                say_into_chat(&message, "Unable to make an embed of search results here.");
                return Err(String::from("Couldn't make an embed of search results."));
            }
            return Ok(());
        } else {
            if let Err(_) = send_error_embed(&message, "There are no results for that query.") {
                say_into_chat(&message, "There are no results for that query.");
            }
            return Err(String::from("Didn't find any results for the request."));
        }
    } else {
        if let Err(_) = send_error_embed(&message, "Couldn't get a response back from the mod portal.") {
            say_into_chat(&message, "Couldn't get a response back from the mod portal.");
        }
        return Err(String::from("Failed to get a response back from the mod portal."));
    }
});
