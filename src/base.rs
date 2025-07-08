use tokio::fs::File;
use tokio::io::AsyncReadExt;
use anyhow::Result;
use chrono::{Local, Datelike, Timelike};
pub fn get_nowtime_str()->String{
    let current_datetime = Local::now();

    let year = current_datetime.year();
    let month = current_datetime.month();
    let day = current_datetime.day();
    let hour = current_datetime.hour();
    let minute = current_datetime.minute();

    let datetime_string = format!("{}-{:02}-{:02} {:02}:{:02}", year, month, day, hour, minute);
    datetime_string

}
pub async fn file_openString(name_file:&str) -> Result<String>{
    let mut file = File::open(name_file).await?;

    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    Ok(contents)
}
