use std::{clone, io::stdin, option, time::Duration};

use reqwest::{header, Response};
use serde_json;

use git2::Repository;

use clap::{arg, Parser};

use indicatif::{ProgressBar, ProgressStyle};

async fn fetch_repo_page(
    user: String,
    page: i32,
    auth_token: String,
) -> Result<Response, reqwest::Error> {
    let url = format!(
        "https://api.github.com/users/{}/repos?page={}&per_page=100",
        user, page
    );

    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().progress_chars("⣾⣽⣻⢿⡿⣟⣯⣷"));
    pb.set_message("Fetching repos...");

    pb.enable_steady_tick(Duration::from_millis(25));

    let response = reqwest::Client::new()
        .get(&url)
        .header("User-Agent", "werdl/gh-clone-all")
        .header(
            "Authorization",
            header::HeaderValue::from_str(&auth_token).unwrap(),
        )
        .header(
            "Accept",
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8",
        )
        .send()
        .await?;

    pb.finish_with_message("Fetched repos");

    Ok(response)
}

async fn fetch_user_repos(
    user: String,
    auth_token: String,
) -> Result<Vec<(String, String)>, reqwest::Error> {
    let mut page = 1;
    let mut repos = Vec::new();
    let mut http_urls = Vec::new();

    let pb = ProgressBar::new_spinner();

    pb.set_style(ProgressStyle::default_spinner().progress_chars("⣾⣽⣻⢿⡿⣟⣯⣷"));
    pb.set_message("Fetching repos (page 1)...");

    loop {
        pb.set_message(format!("Fetching repos (page {})...", page));
        let response = fetch_repo_page(user.clone(), page, auth_token.clone()).await?;

        // check if the request was successful
        if !response.status().is_success() {
            return Err(response.error_for_status().unwrap_err());
        }

        // get the response JSON
        let response_text = response.json::<serde_json::Value>().await.unwrap();

        if response_text.as_array().unwrap().is_empty() {
            break;
        }

        let repos_on_page = response_text.as_array().unwrap().iter().map(|repo| {
            let name = repo["name"].as_str().unwrap();
            name.to_string()
        });

        let http_urls_on_page = response_text.as_array().unwrap().iter().map(|repo| {
            let http_url = repo["html_url"].as_str().unwrap();
            http_url.to_string()
        });

        repos.extend(repos_on_page);
        http_urls.extend(http_urls_on_page);

        // check if there is a next page
        if let Some(_) = fetch_repo_page(user.clone(), page, auth_token.clone())
            .await?
            .headers()
            .get(header::LINK)
        {
            // increment the page number and continue fetching
            page += 1;
            continue;
        }

        // break the loop if there is no next page
        break;
    }

    pb.finish_with_message("Fetched repos");

    Ok(repos
        .into_iter()
        .zip(http_urls.into_iter())
        .collect::<Vec<(String, String)>>())
}

async fn clone_repo(url: String, path: String, pb: &ProgressBar) -> Result<(), reqwest::Error> {
    let repo = match Repository::clone(&url, path) {
        Ok(repo) => repo,
        Err(e) => panic!("failed to clone: {}", e),
    };

    pb.inc(1);

    Ok(())
}

async fn clone_repos(
    repos: Vec<(String, String)>,
    path_prefix: String,
) -> Result<(), reqwest::Error> {
    let pb = ProgressBar::new(repos.len() as u64);
    for (repo, http_url) in repos {
        clone_repo(http_url, format!("{}/{}", path_prefix, repo), &pb);
    }

    pb.finish_with_message("Cloned repos");
    Ok(())
}

#[derive(Parser, Debug)]
struct Options {
    #[arg(short, long, default_value = "werdl")]
    user: String,

    #[arg(short, long, default_value = "./")]
    path_prefix: String,

    #[arg(short, long, default_value = "")]
    auth_token: String,

    #[arg(short, long, default_value = "false")]
    clone: bool,
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let options = Options::parse();

    let user = "werdl".to_string();
    let repos = fetch_user_repos(options.user.clone(), options.auth_token).await?;
    for repo in repos.clone() {
        println!("{:?}", repo);
    }

    println!("Found in total {} repos for {}", repos.len(), options.user);

    if options.clone {
        tokio::spawn(clone_repos(repos, options.path_prefix));
    }

    Ok(())
}
