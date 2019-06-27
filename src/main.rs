use std::collections::HashMap;
use std::path::{Path, PathBuf};

use structopt::StructOpt;

type CliResult = Result<(), String>;
type Redirects = HashMap<PathBuf, String>;

fn main() -> CliResult {
    let cli = Cli::from_args();
    check_input_dir(&cli.input_dir)?;
    check_output_dir(&cli.output_dir)?;

    let redirects = build_redirects(&cli.input_dir, &cli.new_base_url);
    print_redirects(&redirects, &cli.output_dir)
}

/// Given a directory full of HTML files
#[derive(StructOpt)]
struct Cli {
    /// The directory containing HTML files to create redirects for
    input_dir: PathBuf,

    /// The target directory for the redirect files.
    output_dir: PathBuf,

    /// The base URL to use in generating the redirects.
    ///
    /// - Protocol is required; if it is not included, it will be ignored
    /// - Trailing slash is forbidden
    new_base_url: String,
}

fn check_input_dir(input_dir: &Path) -> CliResult {
    if !input_dir.exists() || !input_dir.is_dir() {
        Err(format!(
            "{} does not exist or exists and is a file",
            input_dir.display()
        ))
    } else {
        Ok(())
    }
}

fn check_output_dir(output_dir: &Path) -> CliResult {
    if output_dir.exists() && !output_dir.is_dir() {
        Err(format!(
            "output_dir {} exists and is not a directory",
            output_dir.display()
        ))
    } else {
        Ok(())
    }
}

fn file_paths(in_dir: &Path) -> Vec<PathBuf> {
    std::fs::read_dir(in_dir)
        .expect("you didn't pass a valid directory, dork")
        .map(|entry| entry.expect("every item ought to be legit, yo").path())
        .flat_map(|path| {
            if path.is_dir() {
                file_paths(&path)
            } else {
                vec![path]
            }
        })
        .collect()
}

fn build_redirects(input_dir: &Path, new_base_url: &str) -> Redirects {
    file_paths(input_dir)
        .iter()
        .map(|path| {
            path.strip_prefix(input_dir)
                .expect("paths contain their own parents")
        })
        .fold(HashMap::new(), |mut map, file_name| {
            map.insert(
                file_name.to_path_buf(),
                format!(
                    r#"<meta http-equiv="refresh" content="0; url={base}/{path}">
<link rel="canonical" href="{base}/{path}" />"#,
                    base = new_base_url,
                    path = file_name.display()
                ),
            );
            map
        })
}

fn print_redirects(redirects: &Redirects, output_dir: &Path) -> CliResult {
    for (file_name, redirect) in redirects {
        let path = output_dir.join(&file_name);
        let parent = path.parent().ok_or(format!("path must include parent"))?;

        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| format!("{:?}", e))?;
        }

        std::fs::write(&path, redirect).map_err(|e| format!("{:?}", e))?;
    }

    Ok(())
}