use std::collections::HashMap;
use std::path::{Path, PathBuf};

use structopt::clap::arg_enum;

use structopt::StructOpt;

type CliResult = Result<(), String>;
type Redirects = HashMap<PathBuf, String>;

fn main() -> CliResult {
    let cli = Cli::from_args();
    check_input_dir(&cli.input_dir)?;
    check_output_dir(&cli.output_dir)?;

    let redirects = build_redirects(&cli.input_dir, &cli.new_base_url);

    match cli.style {
        Style::File => print_redirect_files(&redirects, &cli.output_dir),
        Style::Netlify => print_netlify_redirects(&redirects, &cli.output_dir),
    }
}

arg_enum! {
    enum Style {
        File,
        Netlify,
    }
}

/// Given a directory full of HTML files, create HTML redirects for them.
#[derive(StructOpt)]
#[structopt(raw(
    setting = "structopt::clap::AppSettings::ColoredHelp",
    setting = "structopt::clap::AppSettings::ColorAlways",
    setting = "structopt::clap::AppSettings::ArgRequiredElseHelp"
))]
struct Cli {
    /// The directory containing HTML files to create redirects for
    input_dir: PathBuf,

    /// The target directory for the redirects.
    output_dir: PathBuf,

    /// The base URL to use in generating the redirects.
    ///
    /// - Protocol is required; if it is not included, it will be ignored
    /// - Trailing slash is forbidden
    new_base_url: String,

    /// Output style to use: file or Netlify rule set
    #[structopt(raw(possible_values = "&Style::variants()"), case_insensitive = true)]
    style: Style,
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
        .fold(HashMap::new(), |mut map, file_path| {
            map.insert(
                file_path.to_path_buf(),
                format!(
                    "{base}/{path}",
                    base = new_base_url,
                    path = file_path.display()
                ),
            );
            map
        })
}

fn print_redirect_files(redirect: &Redirects, output_dir: &Path) -> CliResult {
    for (file_path, redirect) in redirect {
        let path = output_dir.join(&file_path);
        let parent = path.parent().ok_or(format!("path must include parent"))?;

        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| format!("{:?}", e))?;
        }
        let content = format!(
            r#"<meta http-equiv="refresh" content="0; url={url}">\n<link rel="canonical" href="{url}" />"#,
            url = redirect
        );
        std::fs::write(&path, content).map_err(|e| format!("{:?}", e))?;
    }

    Ok(())
}

fn print_netlify_redirects(redirects: &Redirects, output_dir: &Path) -> CliResult {
    let content = redirects
        .iter()
        .fold(String::new(), |string, (file_path, redirect)| {
            string + "\n" + &format!("/{} {} 301", file_path.display(), redirect)
        });

    let path = output_dir.join("_redirects");
    std::fs::write(&path, &content).map_err(|e| format!("{:?}", e))
}
