use anyhow::{anyhow, Result};
use bulid::AndaConfig;
use execute::Execute;
use regex::Regex;
use std::{
    env, fs,
    io::Write,
    process::{Command, Output, Stdio},
};
mod bulid;

struct Dependency {
    depth: u8,
    name: String,
    version: String,
}

fn main() -> Result<()> {
    // prefix none => easier to parse (no fancy formats)
    println!("Gathering deps");
    let text = run(vec!["cargo", "tree", "--prefix", "depth"])?;
    let text = text.as_str();
    let re = regex::Regex::new(r"(\d+)([\w-]+) v([\d\.]+)( \(.+\))?")?;
    //? https://github.com/rust-lang/cargo/issues/10995
    println!("Will build:");
    let mut deps: Vec<Dependency> = vec![];
    for cap in re.captures_iter(text) {
        if let Some(opt) = &cap.get(4) {
            if opt.as_str() == "*" {
                continue; // duplicates
            }
        }
        let name = cap[2].to_string();
        let version = cap[3].to_string();
        print!("{}[2K\r {} {}", 27 as char, name, version);
        std::io::stdout().flush()?;
        let mut already_exists = false;
        for dep in &deps {
            if dep.name == name {
                already_exists = true;
                break;
            }
        }
        if already_exists {
            continue;
        }
        if !crate_exists(&name, &version)? {
            println!();
            deps.push(Dependency {
                depth: cap[1].parse()?,
                name,
                version,
            });
        }
    }
    deps.sort_by(|a, b| a.depth.cmp(&b.depth).reverse());

    let mut conf = AndaConfig::new();

    println!("{}[2K\r[仕組み]", 27 as char);

    let mut i = 0;
    for dep in &deps {
        i += 1;
        print!(" [{}/{}] Generating specs: {}\r", i, &deps.len(), dep.name);
        std::io::stdout().flush()?;
        if dep.depth == 0 {
            let output = full_run(vec!["rust2rpm", "."], ".")?;
            _gen_spec(&mut conf, dep, output)?;
            break;
        }
        build(&mut conf, dep)?;
    }

    println!("\nGenerating Andaman cfg");
    conf.hcl()?;
    // println!("[もしもし]");
    // run(vec!["anda", "build"])?;
    Ok(())
}

fn _gen_spec(conf: &mut AndaConfig, dep: &Dependency, output: String) -> Result<()> {
    let re = Regex::new(r"Generated: (.+\.spec)")?;
    let cap = re
        .captures(output.as_str())
        .ok_or(anyhow!("No specs generated"))?;
    let specfile = cap
        .get(1)
        .ok_or(anyhow!("rust2rpm parsing error"))?
        .as_str();
    // ! meantime no build_deps
    conf.add(&dep.name, specfile.to_string(), vec![]);
    Ok(())
}

fn run(cmd: Vec<&str>) -> Result<String> {
    Ok(String::from_utf8(_run(cmd, ".")?.stdout)?)
}

fn full_run(cmd: Vec<&str>, dir: &str) -> Result<String> {
    let output = _run(cmd, dir)?;
    Ok(format!(
        "{}\n{}",
        String::from_utf8(output.stdout)?,
        String::from_utf8(output.stderr)?
    ))
}

fn _run(cmd: Vec<&str>, dir: &str) -> Result<Output> {
    let output = Command::new(cmd[0])
        .args(&cmd[1..])
        .current_dir(dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .current_dir(dir)
        .execute_output()?;
    if let Some(rc) = output.status.code() {
        if rc != 0 {
            panic!("Command output was {}: {:?}", rc, cmd);
        }
    } else {
        panic!("Command was interrupted: {:?}", cmd)
    }
    Ok(output)
}

fn crate_exists(name: &String, version: &String) -> Result<bool> {
    let text = run(vec![
        "dnf",
        "repoquery",
        "--whatprovides",
        format!("crate({}) = {}", name, version).as_str(),
    ])?;
    // 危險動作　切勿模仿
    // matches for package names
    let re = Regex::new(r"(?m)^[\w\-:.]+$")?;
    Ok(re.is_match(text.as_str()))
}

fn build(conf: &mut AndaConfig, dep: &Dependency) -> Result<()> {
    let name = &dep.name;
    let version = &dep.version;
    let mut cwd = env::current_dir()?;
    cwd.push(name);
    let cwd = cwd.to_str().ok_or(anyhow!("Can't convert string"))?;
    if !folder_exists(&name)? {
        fs::create_dir(&cwd)?;
    }
    let output = full_run(vec!["rust2rpm", name.as_str(), version.as_str()], cwd)?;
    _gen_spec(conf, dep, output)?;
    Ok(())
}

fn folder_exists(name: &String) -> Result<bool> {
    let mut path = env::current_dir()?;
    path.push(name);
    if let Ok(metadata) = fs::metadata(path) {
        return Ok(metadata.is_dir());
    } else {
        return Ok(false);
    }
}

#[cfg(test)]
mod tests {
    use crate::{build, bulid::AndaConfig};

    #[test]
    fn a() {
        // build(
        //     &mut AndaConfig::new(),
        //     &"reqwest".to_string(),
        //     &"0.11.11".to_string(),
        // )
        // .unwrap();
    }
}
