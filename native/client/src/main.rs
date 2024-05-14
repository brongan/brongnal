use anyhow::Result;
use client::BrongnalUser;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alphanumeric1, multispace1};
use nom::combinator::map;
use nom::sequence::preceded;
use nom::IResult;
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
    let mut user = BrongnalUser::memory_user().await?;

    println!(
        r#"Commands:
        register NAME
        message NAME MESSAGE
        get_messages
        Type Control-D (on Unix) or Control-Z (on Windows)
        to close the connection."#
    );

    let mut command_lines = BufReader::new(stdin()).lines();
    while let Some(command) = command_lines.next() {
        let command = command?;
        let (_, command) = parse_command(&command).map_err(|e| e.to_owned())?;
        let result: Result<()> = match &command {
            Command::Register(name) => user.register(name).await,
            Command::Message(name, message) => user.message(name, message).await,
            Command::GetMessages => user.get_messages().await,
        };
        if let Err(e) = result {
            eprintln!("Command {command:?} failed: {e}");
        }
    }

    Ok(())
}
