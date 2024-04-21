use ::client::{MemoryClient, X3DHClient};
use anyhow::Result;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alphanumeric1, multispace1};
use nom::combinator::map;
use nom::sequence::preceded;
use nom::IResult;
use protocol::x3dh::{x3dh_initiate_recv, x3dh_initiate_send, Message};
use server::service::brongnal_client::BrongnalClient;
use server::service::{
    RegisterPreKeyBundleRequest, RequestPreKeysRequest, RetrieveMessagesRequest, SendMessageRequest,
};
use std::io::{stdin, BufRead, BufReader};

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

#[tokio::main]
async fn main() -> Result<()> {
    let mut stub = BrongnalClient::connect("http://[::1]:8080").await?;
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
        let result: Result<()> = match &command {
            Command::Register(name) => {
                my_name = name.clone();
                {
                    let identity = my_name.clone();
                    let client = &mut x3dh_client;
                    let stub = &mut stub;
                    async move {
                        println!("Registering {identity}!");
                        let ik = client
                            .get_identity_key()?
                            .verifying_key()
                            .as_bytes()
                            .to_vec();
                        let spk = client.get_spk()?;
                        let otk_bundle = client.add_one_time_keys(100);
                        let request = tonic::Request::new(RegisterPreKeyBundleRequest {
                            ik: Some(ik),
                            identity: Some(identity.clone()),
                            spk: Some(spk.into()),
                            otk_bundle: Some(otk_bundle.into()),
                        });
                        stub.register_pre_key_bundle(request).await?;
                        println!("Registered: {identity}!");
                        Ok(())
                    }
                }
                .await
            }
            Command::Message(name, message) => {
                {
                    let recipient_identity = name.clone();
                    let client = &x3dh_client;
                    let stub = &mut stub;
                    let message = message.as_bytes();
                    async move {
                        println!("Messaging {recipient_identity}.");
                        let request = tonic::Request::new(RequestPreKeysRequest {
                            identity: Some(recipient_identity.clone()),
                        });
                        let response = stub.request_pre_keys(request).await?;
                        let (_sk, message) = x3dh_initiate_send(
                            response.into_inner().try_into()?,
                            &client.get_identity_key()?,
                            message,
                        )?;
                        let request = tonic::Request::new(SendMessageRequest {
                            recipient_identity: Some(recipient_identity),
                            message: Some(message.into()),
                        });
                        stub.send_message(request).await?;
                        println!("Message Sent!");
                        Ok(())
                    }
                }
                .await
            }
            Command::GetMessages => {
                {
                    let name = my_name.clone();
                    let client = &mut x3dh_client;
                    let stub = &mut stub;
                    async move {
                        let response = stub
                            .retrieve_messages(RetrieveMessagesRequest {
                                identity: Some(name.clone()),
                            })
                            .await?;
                        let messages = response.into_inner().messages;
                        println!("Retrieved {} messages.", messages.len());
                        for message in messages {
                            let Message {
                                sender_identity_key,
                                ephemeral_key,
                                otk,
                                ciphertext,
                            } = message.try_into()?;
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
                }
                .await
            }
        };
        if let Err(e) = result {
            eprintln!("Command {command:?} failed: {e}");
        }
    }

    Ok(())
}
