use std::{borrow::Borrow, error::Error, fs::File, sync::Arc, time::Duration};

use futures::{lock::Mutex, TryStreamExt};
use hyper::{
    body::{Bytes, HttpBody},
    Client,
};
use tokio::{io::AsyncWriteExt, runtime::Runtime, signal};
use tokio::{net::TcpStream, time::sleep};

use image_data::{FlutInstructions, GifSource};

mod cli;
mod image_data;

fn main() -> Result<(), Box<dyn Error>> {
    let options = cli::get_options();
    println!("🖼️ File: {}", options.file);
    println!("🖥️ URL: {}", options.url);

    // Create Tokio Runtime
    let rt = Runtime::new().unwrap();

    let file_path = options.file.clone();
    let gif = match &file_path {
        n if n.starts_with("http:") || n.starts_with("https:") => rt.block_on(async move {
            let https = hyper_tls::HttpsConnector::new();
            let http_client = Client::builder().build::<_, hyper::Body>(https);
            let res = http_client.get(file_path.parse()?).await?;
            let gif_data = res.into_body();
            let size_hint = gif_data.size_hint();

            let mut data = size_hint
                .exact()
                .or_else(|| size_hint.upper())
                .map(|size| size as usize)
                .map_or_else(Vec::new, Vec::with_capacity);
            let bytes_vec: Vec<Bytes> = gif_data.try_collect().await?;
            for bytes in bytes_vec {
                data.extend(bytes.into_iter());
            }

            println!("Downloaded file");

            Result::<_, Box<dyn Error>>::Ok(GifSource::Vec(data))
        })?,
        _ => GifSource::File(File::open(file_path)?),
    };

    println!("🖼️ Loading image...");

    let image = match gif {
        GifSource::File(file) => image_data::load_image(file),
        GifSource::Vec(vec) => image_data::load_image::<&[u8]>(vec.borrow()),
    };

    println!("Optimizing...");

    let optimized = image_data::optimize_image(image, options.similarity);

    println!("📝 Generating Commands...");

    let commands =
        image_data::optimized_image_to_instructions(optimized, options.offset.0, options.offset.1);

    rt.block_on(fluten(&options.url, commands))?;
    Ok(())
}

async fn fluten(url: &str, commands: FlutInstructions) -> Result<(), Box<dyn Error>> {
    println!("📡 Connecting to server...");
    let mut stream = TcpStream::connect(url).await?;
    println!("🚿 Flut! 🚿");
    stream.write_all(&commands.start).await?;
    stream.flush().await?;
    let stop = Arc::new(Mutex::new(false));
    let stop2 = stop.clone();
    tokio::spawn(async move {
        drop(signal::ctrl_c().await);
        println!("Stopping the Flut...");
        *stop2.lock().await = true;
    });

    loop {
        for (cmds, _corrections, delay) in commands.frames.iter() {
            //let done = Arc::new(Mutex::new(false));
            //let done2 = done.clone();
            let delay2 = *delay;
            //tokio::spawn(async move {
            //    sleep(Duration::from_millis(delay2 as u64 * 10)).await;
            //    *done2.lock().await = true;
            //});
            /*loop {
                stream.write_all(cmds).await?;
                if *done.lock().await {
                    break;
                }
            }*/
            stream.write_all(cmds).await?;
            stream.flush().await?;
            sleep(Duration::from_millis(delay2 as u64 * 10)).await;

            if *stop.lock().await {
                stream.flush().await?;
                stream.shutdown().await?;
                println!("Bye 👋");
                return Ok(());
            }
        }
    }
}
