//! Bounded, save-time presentation only. Never used by input/frame processing.
use super::MAX_FILE_BYTES;
const BORDER: &str = "# ========================================================================";

pub fn for_save(source: &str, updated: String) -> Result<String, String> {
    if source.lines().any(|line| line.trim() == "[universal.settings]") {
        organize(&updated)
    } else {
        Ok(updated)
    }
}

/// Keep property bytes and per-category choice order; only move whole sections.
pub fn organize(source: &str) -> Result<String, String> {
    let lines: Vec<_> = source.lines().collect();
    let mut clean = String::new();
    for (i, line) in lines.iter().enumerate() {
        if *line == BORDER || (line.starts_with("# ---- ") && line.ends_with(" ----"))
            || (i > 0 && i + 1 < lines.len() && lines[i-1] == BORDER && lines[i+1] == BORDER) {
            continue;
        }
        clean.push_str(line);
        clean.push('\n');
    }
    let mut preamble = String::new();
    let mut sections: Vec<(String, String)> = Vec::new();
    for line in clean.lines() {
        let header = line.split('#').next().unwrap_or("").trim();
        if header.starts_with('[') && header.ends_with(']') {
            sections.push((header[1..header.len()-1].to_owned(), String::new()));
        }
        if let Some((_, body)) = sections.last_mut() {
            body.push_str(line); body.push('\n');
        } else { preamble.push_str(line); preamble.push('\n'); }
    }
    let mut fighters = Vec::new();
    for (header, _) in &sections {
        let fighter = header.split('.').next().unwrap_or("");
        if fighter != "universal" && fighter != "template" && !fighters.iter().any(|f| f == fighter) {
            fighters.push(fighter.to_owned());
        }
    }
    let mut output = String::new();
    if !preamble.trim().is_empty() { output.push_str(preamble.trim()); output.push_str("\n\n"); }
    for (header, body) in &sections {
        if header.starts_with("universal.") || header.starts_with("template.") {
            output.push_str("# ---- UNIVERSAL SETTINGS ----\n");
            output.push_str(body.trim()); output.push_str("\n\n");
        }
    }
    for fighter in fighters {
        let title = format!("# {:^70}", fighter.replace('_', " ").to_uppercase());
        output.push_str(&format!("{BORDER}\n{}\n{BORDER}\n\n", title.trim_end()));
        let mut blocks: Vec<_> = sections.iter().filter(|(h,_)| h.split('.').next() == Some(fighter.as_str())).collect();
        // Stable sort preserves each category's menu order and implicit default.
        blocks.sort_by_key(|(h,body)| assist_group(body).map_or(rank(h.split('.').nth(1).unwrap_or("")), |(n,_)| 100+n));
        let mut previous = String::new();
        for (header, body) in blocks {
            let slot = header.split('.').nth(1).unwrap_or("");
            let mode = assist_group(body);
            let group = if slot == "settings" { "FIGHTER SETTINGS".to_owned() }
                else if let Some((_,name)) = mode { format!("ASSIST RULES: {name}") }
                else if ["neutral","jump","roll","attack","drop"].contains(&slot) { format!("LEDGE TRAPS: {}", slot.to_uppercase()) }
                else { format!("BUTTON SEQUENCES: {}", slot.to_uppercase()) };
            if group != previous { output.push_str(&format!("# ---- {group} ----\n")); previous = group; }
            if slot != "settings" {
                let name = body.lines().find_map(|l| l.trim().split_once('=').filter(|(k,_)| k.trim() == "name").map(|(_,v)| v.trim())).unwrap_or(header);
                output.push_str(&format!("# ---- {name} | {header} ----\n"));
            }
            output.push_str(body.trim()); output.push_str("\n\n");
        }
    }
    if output.len() as u64 > MAX_FILE_BYTES {
        // Presentation must not reject an otherwise valid explicit save. A
        // dense roster can fit on disk while generated name/ID banners do not.
        // Keep the edited source verbatim rather than discard authored comments
        // or increase the parser/file limits merely for decorative formatting.
        if source.len() as u64 <= MAX_FILE_BYTES { return Ok(source.to_owned()); }
        return Err("sequence file exceeds 512 KiB".into());
    }
    Ok(output)
}
fn assist_group(body: &str) -> Option<(usize, &'static str)> {
    let token=body.lines().find_map(|line| {
        let (key,value)=line.split('#').next()?.split_once('=')?;
        (key.trim()=="reaction").then(||value.trim().split(':').next().unwrap_or(""))
    })?;
    match token {
        "roll"|"roll_cross"|"roll_vulnerable"|"roll_cross_vulnerable"=>Some((0,"ROLL REACTIONS")),
        "shield_hit"|"shield_hit_vulnerable"=>Some((1,"OUT OF SHIELD")),
        "grab_dodged"|"grab_dodged_vulnerable"=>Some((2,"GRAB DODGE + PUNISH")),
        "ledge_regrab"=>Some((3,"VULNERABLE LEDGE / REGRAB")),
        "auto_roll"|"auto_spot_dodge"|"auto_air_dodge"|"auto_grab"|"auto_shield_grab"|"auto_grab_dodged"|"auto_shield_backup"=>Some((6,"AUTO REACT")),
        "spot_dodge"|"spot_dodge_any"=>Some((4,"SPOT DODGE PUNISH")),
        "shield_pressure_vulnerable"|"enemy_shield"|"shield_roll"|"shield_roll_cross"|"shield_roll_vulnerable"|"shield_roll_cross_vulnerable"|"shield_roll_away"|"shield_roll_away_vulnerable"=>Some((5,"SHIELD PRESSURE")), _=>None,
    }
}
fn rank(slot: &str) -> usize {
    ["settings","neutral","jump","roll","attack","drop","x","y","dpad_up","dpad_down","dpad_left","dpad_right"].iter().position(|s| *s == slot).unwrap_or(99)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stable_grouping_and_repeat_saves() {
        let source = "[universal.settings]\nenabled = true\n[kirby.roll.a]\nname = First\nactions = grab\n[yoshi.neutral.a]\nactions = grab\n[kirby.settings]\nwalk_strength = 0.4\n[kirby.roll.b]\nname = Second\nactions = grab\n";
        let result = organize(source).unwrap();
        assert_eq!(organize(&result).unwrap(), result);
        assert!(result.find("[kirby.settings]").unwrap() < result.find("[kirby.roll.a]").unwrap());
        assert!(result.find("[kirby.roll.a]").unwrap() < result.find("[kirby.roll.b]").unwrap());
        assert!(result.find("[kirby.roll.b]").unwrap() < result.find("[yoshi.neutral.a]").unwrap());
        assert!(result.starts_with("# ---- UNIVERSAL SETTINGS ----"));
    }

    #[test]
    fn decorative_headers_never_reject_a_valid_near_limit_source() {
        let mut source=String::from("[universal.settings]\nenabled=true\n");
        let mut count=0;
        'fighters: for fighter in ["mario","luigi","kirby","yoshi","bowser"] {
            for slot in ["neutral","jump","roll","attack","drop","x","y","dpad_up","dpad_down","dpad_left","dpad_right","l3","r3"] {
                for i in 0..32 {
                    source.push_str(&format!("[{fighter}.{slot}.option_{i}]\nname = A saved option\nstart_delay=0\nactions=up_b:1\n"));
                    count+=1;if count==2040 {break 'fighters;}
                }
            }
        }
        let remaining=MAX_FILE_BYTES as usize-source.len();
        source.push('#');source.extend(std::iter::repeat('x').take(remaining-2));source.push('\n');
        assert_eq!(source.len() as u64,MAX_FILE_BYTES);
        crate::sequence_config::persist::validate_output(&source).unwrap();
        assert_eq!(organize(&source).unwrap(),source);
        assert_eq!(for_save(&source,source.clone()).unwrap(),source);
        source.push('x');assert!(organize(&source).is_err());
    }
}
