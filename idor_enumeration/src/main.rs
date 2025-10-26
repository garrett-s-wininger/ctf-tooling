use anyhow::{bail, Context, Result};
use clap::{Args, Parser};
use reqwest::blocking::Client;
use reqwest::header::{COOKIE, HeaderMap, HeaderValue};
use serde_json::Value;
use std::io;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Arguments {
    #[arg(short, long)]
    endpoint: String,

    #[arg(short, long)]
    selector: String,

    #[clap(flatten)]
    session_auth: SessionAuthentication,
}

#[derive(Args, Debug)]
struct SessionAuthentication {
    #[arg(short = 'n', long = "session-cookie-name")]
    name: Option<String>,

    #[arg(short = 'i', long = "session-id")]
    id: Option<String>,
}

fn main() -> Result<()> {
    let args = Arguments::parse();
    let mut headers = HeaderMap::new();

    if args.session_auth.name.is_some() || args.session_auth.id.is_some() {
        if args.session_auth.name.is_none() || args.session_auth.id.is_none() {
            bail!("Session authentication, when provided, must be complete");
        }

        headers.insert(
            COOKIE,
            HeaderValue::from_str(
                &format!(
                    "{}={}",
                    args.session_auth.name.as_ref().unwrap(),
                    args.session_auth.id.as_ref().unwrap()
                )
            ).with_context(|| "Session information contains invalid characters")?
        );
    }

    let client = Client::builder()
        .default_headers(headers)
        .user_agent("IdorEnumerator/0.1")
        .build()
        .with_context(|| "Failed to create request client")?;

    let mut idx = 1;

    loop {
        let url = format!("{}{}", &args.endpoint, idx);

        let response = client.get(&url)
            .send()
            .with_context(|| format!("Failed to request: {}", url))?;

        let json: Value = response.json()
            .with_context(|| "Failed to read response body")?;

        let pretty_json = serde_json::to_string_pretty(&json)
            .with_context(|| "Failed to prettify JSON response data")?;

        println!("Results for {}:\n", &url);
        println!("{}", pretty_json);

        let mut input = String::new();

        io::stdin().read_line(&mut input)
            .with_context(|| "Failed to read input")?;

        idx += 1;
    }
}
