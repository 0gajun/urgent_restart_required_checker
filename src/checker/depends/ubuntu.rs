use std::process::{Command, Stdio};
use std::fs::File;
use std::io::{Read, Write, ErrorKind};

#[derive(Clone)]
pub struct PkgInfo {
    pub name: String,
    pub current_version: String,
}

#[derive(Clone)]
pub struct Changelog {
    pub version: String,
    pub content: String,
}

#[derive(Clone)]
pub struct UpdateInfo {
    pub pkg_info: PkgInfo,
    pub change_logs: Vec<Changelog>,
}

pub fn is_ubuntu() -> bool {
    if !is_debian() {
        return false;
    }

    File::open("/etc/lsb-release")
        .as_mut()
        .map(contains_ubuntu)
        .unwrap_or(false)
}

// xargs apt changelog < /var/run/reboot-required.pkgs 2>/dev/null | grep urgency=high
// TODO: Fix return type
pub fn get_urgent_updates() -> Result<Option<Vec<UpdateInfo>>, String> {
    let reboot_required_pkgs = try!(get_reboot_required_pkgs());

    let mut update_infos = vec![];
    for ref pkg_info in reboot_required_pkgs {
        if let Some(update_info) = try!(get_urgent_update_info(pkg_info)) {
            update_infos.push(update_info);
        }
    }

    Ok(if update_infos.is_empty() {
        None
    } else {
        Some(update_infos)
    })
}

fn get_urgent_update_info(pkg_info: &PkgInfo) -> Result<Option<UpdateInfo>, String> {
    let output = {
        let apt_output = try!(Command::new("apt")
            .arg("changelog")
            .arg(&pkg_info.name)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .or_else(|_| Err("Cannot execute apt command.".to_string())));

        try!(String::from_utf8(apt_output.stdout).or(Err("Cannot convert output into utf8 format")))
    };

    let mut update_info = UpdateInfo {
        pkg_info: pkg_info.clone(),
        change_logs: vec![],
    };

    let mut iter = output.lines();
    iter.by_ref().skip_while(|x| x.starts_with(&pkg_info.name));

    while let Some(version) = iter.next() {
        if version.contains(&pkg_info.current_version) {
            break;
        }

        let content = iter.by_ref()
            .take_while(|x| x.starts_with(&pkg_info.name))
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join("\n");

        if version.contains("urgency=") && !version.contains("urgency=low") &&
           !version.contains("urgency=medium") {
            update_info.change_logs.push(Changelog {
                version: version.to_string(),
                content: content,
            });
        }
    }
    Ok(if update_info.change_logs.is_empty() {
        None
    } else {
        Some(update_info)
    })
}

fn get_reboot_required_pkgs() -> Result<Vec<PkgInfo>, String> {
    const pkg_file: &'static str = "/var/run/reboot-required.pkgs";
    let mut reboot_required_pkgs = String::new();

    // Read package names
    match File::open(pkg_file).map_err(|e| e.kind()) {
        Ok(ref mut f) => {
            try!(f.read_to_string(&mut reboot_required_pkgs)
                .or(Err("Cannot read content from".to_string())));
        }
        Err(ErrorKind::NotFound) => return Ok(vec![]), // No update exists
        Err(e) => return Err(format!("{:?}", e)),
    }

    // Get versions
    let mut pkg_infos = vec![];
    for pkg in reboot_required_pkgs.lines() {
        pkg_infos.push(PkgInfo {
            name: pkg.to_string(),
            current_version: try!(get_pkg_version(pkg)),
        });
    }

    Ok(pkg_infos)
}

fn get_pkg_version(pkg_name: &str) -> Result<String, String> {
    let dpkg = try!(Command::new("dpkg")
        .arg("-s")
        .arg(pkg_name)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .or_else(|_| Err(format!("Cannot get {}'s version", pkg_name))));

    let output = try!(String::from_utf8(dpkg.stdout)
        .or(Err("Cannot convert output into utf8 format")));

    let mut itr = output.lines().skip_while(|e| !e.contains("Version"));
    let version_line = try!(itr.next()
        .ok_or(format!("Cannot find {}'s version from dpkg", pkg_name)));
    let version = try!(version_line.split_whitespace()
        .skip_while(|e| e.contains("Version"))
        .next()
        .ok_or(format!("Cannot extract {}'s version from dpkg", pkg_name)));

    Ok(version.to_string())
}

fn contains_ubuntu(f: &mut File) -> bool {
    let mut content = String::new();
    f.read_to_string(&mut content)
        .map(|_| content.contains("Ubuntu"))
        .unwrap_or(false)
}

fn is_debian() -> bool {
    match File::open("/etc/debian_version") {
        Ok(_) => true,
        Err(_) => false,
    }
}
