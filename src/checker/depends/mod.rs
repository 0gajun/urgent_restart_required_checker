mod ubuntu;

enum Distro {
    Ubuntu,
    Unknown
}

pub fn get_urgent_updates() -> Option<Vec<String>> {
    match detect_distribution() {
        Some(Distro::Ubuntu) => {
            match ubuntu::get_urgent_updates() {
                Ok(Some(update_infos)) => for ref info in update_infos {
                    println!("[{}]", info.pkg_info.name);
                    let ref change_logs = info.change_logs;
                    for ref cl in change_logs {
                        println!("  change_logs: {}", cl.version);
                    }
                },
                Ok(None) => println!("No updates"),
                Err(e) => println!("Err: {}", e)
            };
            Some(vec!["foobar".to_string()])  
        },
        _ => None
    }
}

fn detect_distribution() -> Option<Distro> {
    if ubuntu::is_ubuntu() {
        return Some(Distro::Ubuntu);
    }

    None
}
