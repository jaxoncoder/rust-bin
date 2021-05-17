use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

fn main() -> std::io::Result<()> {
    let mut desktop_files = HashMap::new();

    for dir in vec![
        "/usr",
        "/var/lib/flatpak/exports",
        "/home/ven/.local",
        "/home/ven/.local/share/flatpak/exports",
    ]
    .iter()
    {
        for path in fs::read_dir(format!("{}/share/applications/", dir).as_str())
            .unwrap()
            .filter_map(Result::ok)
            .map(|p| p.path())
            .filter(|p| match p.extension() {
                Some(ext) => ext == "desktop",
                None => false,
            })
        {
            let stem = path.file_stem().unwrap().to_str().unwrap().to_string();

            let text = fs::read_to_string(path)?;

            if !text.contains("[Desktop Entry]")
                || text.contains("NoDisplay=true")
                || text.contains("Hidden=true")
            {
                continue;
            }

            let start = text.find("Name=").expect("Lol no name") + 5;
            let end = start + text[start..].find('\n').expect("Lol no newline");
            let name = text[start..end].to_string();
            desktop_files.insert(name, stem);
        }
    }

    let colour_str = fs::read_to_string("/home/ven/.cache/wal/colors")?;
    let colours: Vec<&str> = colour_str.split('\n').collect();
    let fg = colours[0];
    let bg = colours[1];

    let cmd = Command::new("dmenu")
        .arg("-i")
        .args(&["-nf", bg, "-nb", fg, "-sf", fg, "-sb", bg])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let mut out = BufReader::new(cmd.stdout.unwrap());

    let mut files = desktop_files.keys().map(|s| &**s).collect::<Vec<_>>();
    files.sort_unstable();

    write!(cmd.stdin.unwrap(), "{}", files.join("\n")).unwrap();

    let mut output = String::new();
    out.read_line(&mut output).expect("bruh");

    if output.is_empty() {
        print!("Cancelled!\n");
    } else {
        let mut cmd = Command::new("i3-msg");
        cmd.arg("exec");
        match desktop_files.get(output.trim()) {
            Some(r) => cmd.arg("gtk-launch").arg(r),
            None => cmd.arg(output.trim()),
        };
        cmd.status().expect("Failed to execute");
    }

    Ok(())
}
