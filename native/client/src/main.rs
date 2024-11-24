use anyhow::Result;
use client::{
    get_keys, get_messages, register_device, register_username, send_message, DecryptedMessage,
    X3DHClient,
};
use nom::character::complete::{alphanumeric1, multispace1};
use nom::IResult;
use proto::gossamer::gossamer_service_client::GossamerServiceClient as GossamerClient;
use proto::service::brongnal_service_client::BrongnalServiceClient as BrongnalClient;
use std::io::stdin;
use std::io::BufRead;
use std::io::BufReader;
use std::str::FromStr;
use std::sync::Arc;
use std::{env, thread};
use tokio::sync::mpsc;
use tokio_rusqlite::Connection;
use tokio_stream::StreamExt;
use tracing::{info, Level};
use tracing_subscriber::filter::Targets;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Debug)]
struct Command {
    to: String,
    msg: String,
}

fn parse_command(input: &str) -> IResult<&str, Command> {
    let (input, name) = alphanumeric1(input)?;
    let (message, _spaces) = multispace1(input)?;
    Ok((
        "",
        Command {
            to: name.to_owned(),
            msg: message.to_owned(),
        },
    ))
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = env::args().collect::<Vec<String>>();
    let name = args.get(1).unwrap().to_owned();
    let addr: String = args
        .get(2)
        .map(|addr| addr.to_owned())
        .unwrap_or("https://signal.brongan.com:443".to_owned());

    let filter = Targets::from_str(std::env::var("RUST_LOG").as_deref().unwrap_or("info"))
        .expect("RUST_LOG should be a valid tracing filter");
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .finish()
        .with(filter)
        .try_init()?;

    let mut brongnal = BrongnalClient::connect(addr.clone()).await?;
    let mut gossamer = GossamerClient::connect(addr.clone()).await?;
    let xdg_dirs = xdg::BaseDirectories::with_prefix("brongnal")?;
    let db_path = xdg_dirs.place_data_file(format!("{}_keys.sqlite", name))?;
    let client = Arc::new(X3DHClient::new(Connection::open(db_path).await?).await?);
    let ik = client.get_ik();

    #[allow(deprecated)]
    let ik_str = base64::encode(ik.verifying_key().as_bytes());
    info!("Registering {name} with key={ik_str} at {addr}");

    register_username(&mut gossamer, ik, name.clone()).await?;
    register_device(&mut brongnal, &client.clone()).await?;

    println!("NAME MESSAGE");

    let (cli_tx, mut cli_rx) = mpsc::unbounded_channel();

    thread::spawn(move || {
        for line in BufReader::new(stdin()).lines() {
            let line = line.unwrap();
            match parse_command(&line).map_err(|e| e.to_owned()) {
                Ok((_, command)) => {
                    if cli_tx.send(command).is_err() {
                        return;
                    }
                }
                Err(e) => eprintln!("Invalid Command: {e}"),
            }
        }
    });

    let messages_stream = get_messages(brongnal.clone(), client.clone());
    tokio::pin!(messages_stream);

    loop {
        tokio::select! {
            command = cli_rx.recv() => {
                match command {
                    Some(command) => {
                        for key in get_keys(&mut gossamer, &command.to).await? {
                            if let Err(e) = send_message(&mut brongnal, &client.clone(), &key, &command.msg)
                            .await {
                                eprintln!("Failed to send message: {e}");
                            }
                        }
                    },
                    None => {
                        eprintln!("Closing...");
                        return Ok(());
                    }
                }

            },
            msg = messages_stream.next() => {
                match msg {
                    Some(Ok(DecryptedMessage { message })) => {
                        println!("Received message from unknown: \"{}\"", String::from_utf8(message).unwrap());
                    },
                    Some(Err(e)) => {
                        eprintln!("Failed to receive decrypted message: {e}");
                    }
                    None =>  {
                        eprintln!("Server terminated connection.");
                        return Ok(())
                    },
                }
            }
        }
    }
}
