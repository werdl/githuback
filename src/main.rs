use std::io::stdin;

use reqwest::{header, Response};
use serde_json;

use git2::Repository;

async fn fetch_repo_page(
    user: String,
    page: i32,
    auth_token: String,
) -> Result<Response, reqwest::Error> {
    let url = format!(
        "https://api.github.com/users/{}/repos?page={}&per_page=100",
        user, page
    );

    println!("Fetching {}", url);
    let client = reqwest::Client::new();
    let response = reqwest::Client::new()
        .get(&url)
        .header("User-Agent", "werdl/gh-clone-all")
        // .header(
        //     "Authorization",
        //     header::HeaderValue::from_str(&auth_token).unwrap(),
        // )
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8")
        .send()
        .await?;

    Ok(response)
}

async fn fetch_user_repos(
    user: String,
    auth_token: String,
) -> Result<Vec<(String, String)>, reqwest::Error> {
    let mut page = 1;
    let mut repos = Vec::new();
    let mut http_urls = Vec::new();
    loop {
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
        if let Some(link_header) = fetch_repo_page(user.clone(), page, auth_token.clone())
            .await?
            .headers()
            .get(header::LINK)
        {
            let link_header = link_header.to_str().unwrap();
            // increment the page number and continue fetching
            page += 1;
            continue;
        }

        // break the loop if there is no next page
        break;
    }
    Ok((repos
        .into_iter()
        .zip(http_urls.into_iter())
        .collect::<Vec<(String, String)>>()))
}

async fn clone_repos(repos: Vec<(String, String)>, path_prefix: String) -> Result<(), reqwest::Error> {
    for (repo, http_url) in repos {
        println!("Cloning {} from {}", repo, http_url);
        let repo = match Repository::clone(&http_url, format!("{}/{}", path_prefix, repo)) {
            Ok(repo) => repo,
            Err(e) => panic!("failed to clone: {}", e),
        };

        println!("Cloned {} to {:?}", http_url, repo.path());
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    println!("Please enter your GitHub token:");
    let mut auth_token = String::new();
    let _ = stdin().read_line(&mut auth_token).unwrap();

    auth_token = auth_token.trim().to_string();

    let user = "werdl".to_string();
    let repos = fetch_user_repos(user.clone(), auth_token).await?;
    for repo in repos.clone() {
        println!("{:?}", repo);
    }

    println!("Found in total {} repos for {}", repos.len(), user);
    Ok(())
}
