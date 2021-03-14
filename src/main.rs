use std::{
    error::Error,
    fs::File,
    io::{Seek, SeekFrom, Write},
    sync::Arc,
    time::Duration,
};

use futures::{lock::Mutex, TryStreamExt};
use gif;
use hyper::Client;
use tokio::{io::AsyncWriteExt, runtime::Runtime, signal};
use tokio::{net::TcpStream, time::sleep};

mod cli;

fn main() -> Result<(), Box<dyn Error>> {
    let options = cli::get_options();
    println!("🖼️ File: {}", options.file);
    println!("🖥️ URL: {}", options.url);

    // Create Tokio Runtime
    let rt = Runtime::new().unwrap();

    let file_path = options.file.clone();
    let file = match &file_path {
        n if n.starts_with("http:") || n.starts_with("https:") => rt.block_on(async move {
            let https = hyper_tls::HttpsConnector::new();
            let http_client = Client::builder().build::<_, hyper::Body>(https);
            let res = http_client.get(file_path.parse()?).await?;
            let gif_data = res.into_body();

            let mut std_temp = tempfile::tempfile()?;
            let temp = Arc::new(Mutex::new(tokio::fs::File::from(std_temp.try_clone()?)));
            gif_data
                .try_for_each(|data| {
                    let temp2 = temp.clone();
                    async move {
                        temp2
                            .lock()
                            .await
                            .write_all(&data)
                            .await
                            .expect("Failed to save GIF");
                        return Ok(());
                    }
                })
                .await?;
            temp.lock().await.flush().await?;
            std_temp.seek(SeekFrom::Start(0))?;
            println!("Downloaded file");

            Result::<_, Box<dyn Error>>::Ok(std_temp)
        })?,
        _ => File::open(file_path)?,
    };

    let decode_options = {
        let mut opt = gif::DecodeOptions::new();
        opt.set_color_output(gif::ColorOutput::Indexed);

        opt
    };

    println!("📝 Generating Commands...");

    // Reserve 8GiB
    let (commands, delays) = {
        let mut commands: Vec<Vec<u8>> = Vec::new();
        let mut delays: Vec<u16> = Vec::new();
        let mut gif_decoder = decode_options.read_info(file).unwrap();

        let global_palette = gif_decoder.global_palette().map(|p| Vec::from(p));

        while let Some(frame) = gif_decoder.read_next_frame().unwrap() {
            let offset = (
                frame.left as u32 + options.offset.0,
                frame.top as u32 + options.offset.1,
            );
            let transparent = frame.transparent;
            let mut frame_commands = Vec::with_capacity(16 * 1024);

            let size = (frame.width as u32, frame.height as u32);

            let palette: &[u8] = frame
                .palette
                .as_ref()
                .unwrap_or_else(|| &global_palette.as_ref().unwrap());

            for (i, &byte) in frame.buffer.iter().enumerate() {
                use std::convert::TryInto;
                let i: u32 = i.try_into().unwrap();
                let pixel = ((i % size.0) + offset.0, i / size.0 + offset.1);
                if transparent.map_or(false, |t| t == byte) {
                    continue;
                }
                let color: [u8; 6] = {
                    let idx = byte as usize * 3;
                    let rgb: [u8; 3] = palette[idx..(idx + 3)].try_into().unwrap();
                    format!("{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2])
                        .as_bytes()
                        .try_into()
                        .unwrap()
                };
                frame_commands.extend_from_slice(format!("PX {} {} ", pixel.0, pixel.1).as_bytes());
                frame_commands.extend_from_slice(&color);
                frame_commands.push(b'\n');
            }
            delays.push(frame.delay);

            commands.push(frame_commands);
        }
        (commands, delays)
    };

    rt.block_on(fluten(&options.url, &commands, &delays))?;
    Ok(())
}

async fn fluten(url: &str, commands: &[Vec<u8>], delays: &[u16]) -> Result<(), Box<dyn Error>> {
    println!("📡 Connecting to server...");
    let mut stream = TcpStream::connect(url).await?;
    println!("🚿 Flut! 🚿");
    let stop = Arc::new(Mutex::new(false));
    let stop2 = stop.clone();
    tokio::spawn(async move {
        drop(signal::ctrl_c().await);
        println!("Stopping the Flut...");
        *stop2.lock().await = true;
    });
    loop {
        for (cmds, &delay) in commands.iter().zip(delays) {
            let done = Arc::new(Mutex::new(false));
            let done2 = done.clone();
            tokio::spawn(async move {
                sleep(Duration::from_millis((33 * delay) as u64)).await;
                *done2.lock().await = true;
            });
            loop {
                stream.write_all(cmds).await?;
                if *done.lock().await {
                    break;
                }
            }
            stream.flush().await?;

            if *stop.lock().await {
                println!("Bye 👋");
                return Ok(());
            }
        }
    }
}
