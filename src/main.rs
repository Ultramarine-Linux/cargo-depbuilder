use execute::Execute;
use std::error::Error;
use std::process::{Command, Stdio};

struct Dependency {
    depth: u8,
    name: String,
    version: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    // prefix none => easier to parse (no fancy formats)
    let output = Command::new("cargo")
        .args(["tree", "--prefix", "depth"])
        .stdout(Stdio::piped())
        // .stderr(Stdio::piped())
        .execute_output()?;

    if let Some(rc) = output.status.code() {
        if rc == 0 {
            println!("Ok.");
        } else {
            eprintln!("Failed.");
        }
    } else {
        eprintln!("Interrupted!");
    }

    let text = String::from_utf8(output.stdout)?;
    let text = text.as_str();
    let re = regex::Regex::new(r"(\d+)([\w-]+) v([\d\.]+)( \(.+\))?")?;
    //? https://github.com/rust-lang/cargo/issues/10995
    let mut deps = vec![];
    for cap in re.captures_iter(text) {
        if let Some(opt) = &cap.get(4) {
            if opt.as_str() == "*" {
                continue;  // duplicates
            }
        }
        deps.push(Dependency {
            depth: cap[1].parse()?,
            name: cap[2].to_string(),
            version: cap[3].to_string(),
        });
    }
    deps.sort_by(|a, b| a.depth.cmp(&b.depth).reverse());

    for dep in deps {
        println!("{}", dep.depth);
        build(dep.name, dep.version)?;
    }
    Ok(())
}

fn build(name: String, version: String) -> Result<(), Box<dyn Error>> {
    Ok(())
}
