use reqwest::Client;
use serde_json::Value;
use csv::Writer;
use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let api_key = env::var("YOUTUBE_API_KEY").expect("YOUTUBE_API_KEY must be set");
    let channel_id = "UCBRxDSTfr2aJVODDh4WG_7g";

    match fetch_videos(&api_key, channel_id).await {
        Ok(videos) => {
            println!("Fetched {} videos", videos.len());

            if videos.is_empty() {
                println!("No videos found");
            } else {
                write_to_csv(videos);
                println!("Videos written to CSV");
            }
        }

        Err(e) => {
            println!("Error fetching videos: {}", e);
            if let Some(reqwest_err) = e.downcast_ref::<reqwest::Error>() {
                if let Some(status) = reqwest_err.status(){
                    println!("HTTP Status {}", status);
                }
            }
        }
    }

    Ok(())
}

//fetch videos
async fn fetch_videos(api_key: &str, channel_id: &str) -> Result<Vec<Value>, Box<dyn std::error::Error>>{
    let client = Client::new();
    let mut videos = Vec::new();
    let mut page_token = String::new();

    loop {
        let url = format!(
            "https://www.googleapis.com/youtube/v3/search?key={}&channelId={}&part=snippet,id&order=date&maxResults=50&type=video&pageToken={}",
            api_key, channel_id, page_token
        );

        let response = client.get(&url).send().await?;

        if !response.status().is_success() {
            println!("API request failed with status: {}", response.status());
            println!("Response body: {}", response.text().await?);
            return Err("API request failed".into());
        }

        let json: Value = response.json().await?;

        if let Some(error) = json.get("error"){
            print!("API returned an error: {:?}", error);
            return Err("API returned an error".into());
        }

        if let Some(items) = json["items"].as_array() {
            videos.extend(items.clone());
        }

        if let Some(next_page_token) = json["nextPageToken"].as_str() {
            page_token = next_page_token.to_string();
        } else {
            break;
        }
    }

    Ok(videos)
}


//write rto CSV function
fn write_to_csv(videos: Vec<Value>) -> Result<(), Box<dyn std::error::Error>>{
    let mut wtr = Writer::from_path("francesco_ciulla_videos.csv")?;

    wtr.write_record(&["Video ID", "Title", "Descrption", "Published At"])?;

    for video in videos {
        let snippet = &video["snippet"];

        wtr.write_record(&[
            video["id"]["videoId"].as_str().unwrap_or(""),
            snippet["title"].as_str().unwrap_or(""),
            snippet["description"].as_str().unwrap_or(""),
            snippet["publishedAt"].as_str().unwrap_or(""),
        ])?;
    }

    wtr.flush()?;
    Ok(())
}