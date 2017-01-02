use std::process::{Command, Stdio};
use std::fs::File;
use std::io::{Read, Write, ErrorKind};

struct PkgInfo {
    name: String,
    current_version: String,
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
pub fn get_urgent_updates() -> Result<Vec<String>, String> {
    let reboot_required_pkgs = try!(get_reboot_required_pkgs());

    let mut xargs = try!(Command::new("xargs")
        .arg("apt")
        .arg("changelog")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .or_else(|_| Err("Cannot execute apt command.".to_string())));

    {
        let ref mut xargs_stdin = try!(xargs.stdin
            .as_mut()
            .ok_or("Cannot open child stdin".to_string()));

        for pkg in reboot_required_pkgs {
            xargs_stdin.write_all((pkg.name + "\n").into_bytes().as_slice());
        }
    }

    let output = {
        let xargs_raw_output = try!(xargs.wait_with_output()
                .or(Err("Cannot get output from apt command")))
            .stdout;

        try!(String::from_utf8(xargs_raw_output).or(Err("Cannot convert output into utf8 format")))
    };

    for line in output.lines() {
        if line.contains("urgency=") && !line.contains("urgency=low") &&
           !line.contains("urgency=medium") {
            println!("{}", line);
        }
    }

    Err("Not implemented".to_string())
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
