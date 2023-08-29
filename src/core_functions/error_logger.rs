use chrono::Local;
use tokio::{fs, io::AsyncWriteExt};

pub async fn error_logger(message: &str) {
    let file_name = Local::now().date_naive().to_string();
    let time = Local::now().time().to_string() + " ";
    println!("Error: {}", message);
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(format!("{}.log", file_name))
        .await
        .unwrap();
    file.write(time.as_bytes()).await.unwrap();
    file.write(message.as_bytes()).await.unwrap();
    file.write("\n".as_bytes()).await.unwrap();
}
