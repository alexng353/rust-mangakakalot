use regex::Regex;
use reqwest::header;
use std::{fs, io::Write};
use tokio;

// global variables
const IMG_DELAY: u64 = 500;
const OUTPUT_DIR: &str = "./output";
const CHAPTER_DELAY: u64 = 3000;

// object with ansii color codes
struct Color {
    red: &'static str,
    green: &'static str,
    yellow: &'static str,
    blue: &'static str,
    magenta: &'static str,
    cyan: &'static str,
    end: &'static str,
}

impl Color {
    fn new() -> Color {
        Color {
            red: "\x1b[31m",
            green: "\x1b[32m",
            yellow: "\x1b[33m",
            blue: "\x1b[34m",
            magenta: "\x1b[35m",
            cyan: "\x1b[36m",
            end: "\x1b[0m",
        }
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // get cmd line args
    let c = Color::new();

    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 {
        println!("{}{}{}", c.red, "No url given", c.end);
        return Ok(());
    }
    let url = &args[1];

    // search args for --skip [number]
    let mut iter = args.iter();

    let mut skip = 0;

    while let Some(arg) = iter.next() {
        if arg == "--skip" {
            if let Some(num) = iter.next() {
                skip = num.parse::<u32>().unwrap();
            }
        }
    }

    println!(
        "{}{} {}{} {}{} {}{} {}",
        c.cyan,
        "rust-mangakakalot",
        c.magenta,
        "v0.1.0",
        c.blue,
        "by",
        c.yellow,
        "alexng353",
        c.end
    );

    // make sure url matches https://mangakakalot.com/read-{something}
    // let re = Regex::new(r"https://mangakakalot.com/[a-zA-Z0-9]+").unwrap();
    // new regex that includes https://chapmanganato.com/manga-oa952283
    let re = Regex::new(r"https://(mangakakalot).com/[a-zA-Z0-9_-]+").unwrap();

    // split url into parts
    let url_parts: Vec<&str> = url.split('/').collect();

    let site_name = url_parts[2];

    if site_name != "mangakakalot.com" {
        println!("{}{}{}{}", c.red, site_name, " is not supported", c.end);
        return Ok(());
    }

    println!("{}{}{}{}", "Site name: ", c.green, site_name, c.end);

    if !re.is_match(url) {
        println!("{}{}{}", c.red, "Invalid url", c.end);
        return Ok(());
    }

    // let res = reqwest::get("https://mangakakalot.com/read-wg9rm158504883358").await; // this is the test url (that works 100%)
    let res = reqwest::get(url).await;
    let html = res.unwrap().text().await.unwrap();

    let re = Regex::new(r#"<h1>(.*)</h1>"#).unwrap();
    let title = re.captures(&html).unwrap().get(1).unwrap().as_str();

    println!("Title: {}{}{}", c.green, title, c.end);

    // println!("{:?}", url_parts);

    // if url[-2] == manga, println!("{}{}{}", c.red, "This is a manga, not a manhwa", c.end);

    // if url.split("/").collect::<Vec<&str>>()[3] == "manga" {
    //     println!("{}{}{}", c.red, "This is a new url", c.end);
    //     return Ok(());
    // }

    // println!("{}", url.split("/").collect::<Vec<&str>>()[4]);

    if !fs::metadata(OUTPUT_DIR).is_ok() {
        fs::create_dir(OUTPUT_DIR)?;
    }
    let tmp: String;
    if site_name == "mangakakalot.com" {
        if url.split("/").collect::<Vec<&str>>()[3] == "manga" {
            tmp = format!(
                r#""https://{}.com/chapter/{}/chapter_[0-9]*\.?[0-9]?""#,
                site_name,
                url.split("/").collect::<Vec<&str>>()[4]
            );
        } else {
            tmp = format!(
                r#""https://{}.com/chapter/{}/chapter_[0-9]*\.?[0-9]?""#,
                site_name,
                title.to_lowercase()
            );
        }
    } else {
        tmp = format!(
            r#""https://{}/{}/chapter-[0-9]*\.?[0-9]?""#,
            site_name,
            url.split("/").collect::<Vec<&str>>()[3]
        );
    }
    println!("{}", tmp);

    let re = Regex::new(&tmp).unwrap();

    // grab the first link, seperate by / and grab the second last element
    let matches = re.find_iter(&html);

    let mut urls = Vec::new();
    for m in matches {
        urls.push(&m.as_str()[1..m.as_str().len() - 1]);
    }

    println!(
        "Found {}{:?}{} chapters",
        c.green,
        urls.clone().len(),
        c.end
    );

    // reverse urls
    urls.reverse();

    // create a ./output folder if it doesn't exist
    fs::create_dir_all(&OUTPUT_DIR)?;

    // time it
    let start = std::time::Instant::now();

    // same code but skip the first {skip} chapters
    for (i, url) in urls.iter().enumerate() {
        if i < skip as usize {
            continue;
        }
        let tmp: String;
        if site_name == "mangakakalot.com" {
            tmp = "_([0-9]+\\.?[0-9]?)".to_string()
        } else {
            tmp = "-([0-9]+\\.?[0-9]?)".to_string()
        }
        let re = Regex::new(&tmp).unwrap();
        let chapter = re.find(url).unwrap().as_str();
        println!(
            "\nDownloading Chapter {}{}{} ({}/{}){}",
            c.green,
            &chapter[1..chapter.len()],
            c.yellow,
            i + 1,
            urls.len(),
            c.end
        );
        get_imgs(url, &format!("{}/chapter{}", &OUTPUT_DIR, chapter)).await;
        tokio::time::sleep(std::time::Duration::from_millis(CHAPTER_DELAY.clone())).await;
    }

    let duration = start.elapsed();
    // println!("{}", duration.as_secs());

    println!(
        "{}{} ({}{} seconds) {}",
        c.green,
        "Done",
        c.cyan,
        duration.as_secs(),
        c.end
    );
    Ok(())
}

async fn get_imgs(url: &str, path: &str) {
    // make chapter folder
    fs::create_dir_all(path).unwrap();

    let res = reqwest::get(url).await;
    let html = res.unwrap().text().await.unwrap();

    let url_parts: Vec<&str> = url.split('/').collect();

    let site_name = url_parts[2];
    let re: Regex;
    if site_name == "mangakakalot.com" {
        re = Regex::new(
            r#"<div class="container-chapter-reader">((.|\n)*)<div style="text-align:center;">"#,
        )
        .unwrap();
    } else {
        // <div class="panel-story-chapter-list">((.|\n)*)<div class="panel-story-comment panel-fb-comment a-h">
        re = Regex::new(r#"<ul class="row-content-chapter">((.|\n)*)</ul>"#).unwrap();
    }

    // save html to file test.htm

    let mut file = fs::File::create("test.htm").unwrap();
    file.write_all(html.as_bytes()).unwrap();

    println!("{}", re);

    let test = re.find_iter(html.as_str()).collect::<Vec<_>>();

    println!("{:?}", test);

    let html = re.captures(&html).unwrap().get(1).unwrap().as_str();

    let re = Regex::new(r#"<img src="([^"]*)"#).unwrap();

    let matches = re.find_iter(&html);

    let mut urls = Vec::new();
    for m in matches {
        urls.push(&m.as_str()[10..m.as_str().len()]);
    }

    println!(
        "Found {}{:?}{} images",
        Color::new().green,
        urls.clone().len(),
        Color::new().end
    );

    // get an image every 500 millis
    let mut i = 0;
    for url in urls.clone() {
        fetch_img(url, &i.to_string(), &path).await;
        i += 1;
        tokio::time::sleep(std::time::Duration::from_millis(IMG_DELAY.clone())).await;
    }
}

async fn fetch_img(url: &str, name: &str, path: &str) {
    let client = reqwest::Client::new();

    // Headers need to be here to trick the server into thinking we are a browser requesting from "https://mangakakalot.com/"
    let res = client
        .get(url)
        .header(header::USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/107.0.0.0 Safari/537.36")
        .header(header::REFERER, "https://mangakakalot.com/")
        .send()
        .await
        .unwrap();

    let num = format!("{:0>3}", name);
    let mut file = fs::File::create(format!("{}/{}.jpg", path, num)).unwrap();
    file.write_all(&res.bytes().await.unwrap()).unwrap();
}
