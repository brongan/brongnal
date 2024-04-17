use ::client::{MemoryClient, X3DHClient};
use anyhow::Result;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alphanumeric1, multispace1};
use nom::combinator::map;
use nom::sequence::preceded;
use nom::IResult;
use protocol::x3dh::{x3dh_initiate_recv, x3dh_initiate_send, Message};
use rustls::pki_types::ServerName;
use server::X3DHServerClient;
use std::io::{stdin, BufRead, BufReader};
use tarpc::tokio_serde::formats::Bincode;
use tarpc::{client, context};
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;
use tokio_rustls::TlsConnector;

async fn connect_tcp(domain: String, port: u16) -> Result<TlsStream<TcpStream>, std::io::Error> {
    use std::sync::Arc;
    let host = format!("{}:{}", &domain, port);

    let root_store =
        rustls::RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    let connector = TlsConnector::from(Arc::new(config));
    let servername = ServerName::try_from(domain).unwrap();

    let stream = TcpStream::connect(host).await?;
    connector.connect(servername, stream).await
}

#[derive(Debug)]
enum Command {
    Register(String),
    Message(String, String),
    GetMessages,
}

fn parse_name(input: &str) -> IResult<&str, &str> {
    alphanumeric1(input)
}

fn parse_register_command(input: &str) -> IResult<&str, Command> {
    let (input, _) = preceded(tag("register"), multispace1)(input)?;
    map(parse_name, |name| Command::Register(name.to_owned()))(input)
}

fn parse_message_command(input: &str) -> IResult<&str, Command> {
    eprintln!("Parsing: {input}");
    let (input, _) = preceded(tag("message"), multispace1)(input)?;
    let (input, name) = parse_name(input)?;
    let (message, _) = multispace1(input)?;
    Ok((&"", Command::Message(name.to_owned(), message.to_owned())))
}

fn parse_get_messages_command(input: &str) -> IResult<&str, Command> {
    map(tag("get_messages"), |_| Command::GetMessages)(input)
}

fn parse_command(input: &str) -> IResult<&str, Command> {
    alt((
        parse_register_command,
        parse_message_command,
        parse_get_messages_command,
    ))(input)
}

async fn register(
    identity: String,
    client: &mut dyn X3DHClient,
    stub: &X3DHServerClient,
) -> Result<()> {
    println!("Registering {identity}!");
    let ik = client.get_identity_key()?.verifying_key();
    let spk = client.get_spk()?;
    let otk_bundle = client.add_one_time_keys(100);
    stub.set_spk(context::current(), identity.clone(), ik, spk)
        .await??;
    stub.publish_otk_bundle(context::current(), identity.clone(), ik, otk_bundle)
        .await??;
    println!("Registered: {identity}!");
    Ok(())
}

async fn send_message(
    recipient_identity: String,
    client: &dyn X3DHClient,
    stub: &X3DHServerClient,
    message: &[u8],
) -> Result<()> {
    println!("Messaging {recipient_identity}.");
    let prekey_bundle = stub
        .fetch_prekey_bundle(context::current(), recipient_identity.clone())
        .await??;
    let (_sk, message) = x3dh_initiate_send(prekey_bundle, &client.get_identity_key()?, message)?;
    stub.send_message(context::current(), recipient_identity, message)
        .await??;
    println!("Message Sent!");
    Ok(())
}

async fn get_messages(
    name: String,
    client: &mut dyn X3DHClient,
    stub: &X3DHServerClient,
) -> Result<()> {
    let messages = stub.retrieve_messages(context::current(), name).await?;
    println!("Retrieved {} messages.", messages.len());
    for Message {
        sender_identity_key,
        ephemeral_key,
        otk,
        ciphertext,
    } in messages
    {
        let otk = if let Some(otk) = otk {
            Some(client.fetch_wipe_one_time_secret_key(&otk)?)
        } else {
            None
        };
        let (_sk, message) = x3dh_initiate_recv(
            &client.get_identity_key()?,
            &client.get_pre_key()?,
            &sender_identity_key,
            ephemeral_key,
            otk,
            &ciphertext,
        )?;
        let message = String::from_utf8(message)?;
        println!("{message}");
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let stream = connect_tcp("brongnal.brongan.com".to_string(), 8080).await?;
    let transport = tarpc::serde_transport::Transport::from((stream, Bincode::default()));
    let stub = X3DHServerClient::new(client::Config::default(), transport).spawn();

    let mut x3dh_client = MemoryClient::new();

    println!(
        r#"Commands:
             register NAME
             message NAME MESSAGE
             get_messages
             Type Control-D (on Unix) or Control-Z (on Windows)
             to close the connection."#
    );
    let mut my_name = "".to_string();

    let mut command_lines = BufReader::new(stdin()).lines();
    while let Some(command) = command_lines.next() {
        let command = command?;
        let (_, command) = parse_command(&command).map_err(|e| e.to_owned())?;
        let result = match &command {
            Command::Register(name) => {
                my_name = name.clone();
                register(my_name.clone(), &mut x3dh_client, &stub).await
            }
            Command::Message(name, message) => {
                send_message(name.clone(), &x3dh_client, &stub, message.as_bytes()).await
            }
            Command::GetMessages => get_messages(my_name.clone(), &mut x3dh_client, &stub).await,
        };
        if let Err(e) = result {
            eprintln!("Command {command:?} failed: {e}");
        }
    }

    Ok(())
}
