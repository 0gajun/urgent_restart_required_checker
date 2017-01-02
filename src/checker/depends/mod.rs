mod ubuntu;

enum Distro {
    Ubuntu,
    Unknown
}

pub fn get_urgent_updates() -> Option<Vec<String>> {
    match detect_distribution() {
        Some(Distro::Ubuntu) => {
            match ubuntu::get_urgent_updates() {
                Ok(x) => println!("Ok"),
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
