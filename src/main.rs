use reqwest::{header, Response};
use serde_json;

async fn fetch_repo_page(user: String, page: i32) -> Result<Response, reqwest::Error> {
    let url = format!("https://api.github.com/users/{}/repos?page={}", user, page);
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header(header::USER_AGENT, "reqwest")
        .send()
        .await?;
    Ok(response)
}

async fn fetch_repo_names_and_urls(user: String) -> Result<Vec<(String, String)>, reqwest::Error> {
    let response = fetch_repo_page(user.clone(), 1).await?;

    // check if the request was successful
    if !response.status().is_success() {
        return Err(response.error_for_status().unwrap_err());
    }

    // get the reponse JSON
    let response_text = response.json::<serde_json::Value>().await.unwrap();

    let repos = response_text.as_array().unwrap().iter().map(|repo| {
        let name = repo["name"].as_str().unwrap();
        let url = repo["html_url"].as_str().unwrap();
        (name.to_string(), url.to_string())
    });

    Ok(repos.collect())
}

async fn fetch_user_repos(user: String) -> Result<Vec<String>, reqwest::Error> {
    let mut page = 1;
    let mut repos = Vec::new();
    loop {
        let response = fetch_repo_page(user.clone(), page).await?;

        // check if the request was successful
        if !response.status().is_success() {
            return Err(response.error_for_status().unwrap_err());
        }

        // get the response JSON
        let response_text = response.json::<serde_json::Value>().await.unwrap();

        let repos_on_page = response_text.as_array().unwrap().iter().map(|repo| {
            let name = repo["name"].as_str().unwrap();
            name.to_string()
        });

        repos.extend(repos_on_page);

        // check if there is a next page
        if let Some(link_header) = fetch_repo_page(user.clone(), page)
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
    Ok(repos)
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let user
        = "rust-lang".to_string();
    let repos = fetch_user_repos(user).await?;
    for repo in repos {
        println!("{}", repo);
    }
    Ok(())
}