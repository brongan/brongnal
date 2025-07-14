use anyhow::Result;
use client::client::MessageModel;
use client::{User, X3DHClient};
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

    let brongnal = BrongnalClient::connect(addr.clone()).await?;
    let gossamer = GossamerClient::connect(addr.clone()).await?;
    let xdg_dirs = xdg::BaseDirectories::with_prefix("brongnal")?;
    let db_path = xdg_dirs.place_data_file(format!("{}_keys.sqlite", name))?;
    let connection = Connection::open(db_path).await?;
    let client = Arc::new(X3DHClient::new(connection.clone()).await?);
    let ik = client.get_ik();

    #[allow(deprecated)]
    let ik_str = base64::encode(ik.verifying_key().as_bytes());
    info!("Registering {name} with key={ik_str} at {addr}");
    let user = User::new(brongnal, gossamer, client, name.clone(), None).await?;
    let history = user.get_message_history().await.unwrap();
    for message in history.messages {
        println!("{message}");
    }

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

    let subscriber = user.get_messages().await?;
    let message_stream = subscriber.into_stream();
    tokio::pin!(message_stream);

    loop {
        tokio::select! {
            command = cli_rx.recv() => {
                match command {
                    Some(command) => {
                        if let Err(e) = user.send_message(command.to, command.msg).await {
                                eprintln!("Failed to send message: {e}");
                        }
                    },
                    None => {
                        eprintln!("Closing...");
                        return Ok(());
                    }
                }

            },
            msg = message_stream.next() => {
                match msg {
                    Some(Ok(MessageModel {sender,text, receiver, db_recv_time, state })) => {
                        println!("Received message from {sender}: {text}");
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
