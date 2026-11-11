use std::collections::HashMap;
use std::io::{self, BufRead, Read, Write};

/*
 TODO:Reply und BusisnessLogik auseinander nehmen...
Damit wir das Hash Struct nicht überall mitpassen müssen

 Datenbank Struct erstellen... das dann ADD etc hat..
 Database
 └── HashMap<String, String>

CommandHandler
 └── Executes commands against Database

Reply
 └── Encodes responses into RESP

*/

enum Reply {
    Pong,
    SimpleString(String),
    BulkString(String),
    Error(CommandError),
    Number(isize),
    NullBulkString(String),
}

enum CommandError {
    UnknownCommand(String),
    WrongNumberOfArguments(String),
}

impl CommandError {
    fn encode(&self) -> String {
        match self {
            Self::UnknownCommand(cmd) => {
                format!("-ERR unknown command '{}'\r\n", cmd)
            }
            Self::WrongNumberOfArguments(cmd) => {
                format!("-ERR wrong number of arguments for '{}' command\r\n", cmd)
            }
        }
    }
}

impl Reply {
    fn encode(&self) -> String {
        match self {
            Self::Pong => String::from("+PONG\r\n"),
            Self::SimpleString(data) => {
                format!("+{}\r\n", data)
            }
            Self::Error(data) => data.encode(),
            Self::Number(data) => {
                format!(":{}\r\n", data)
            }
            Self::BulkString(data) => {
                format!("${}\r\n{}\r\n", data.len(), data)
            }
            Self::NullBulkString(data) => {
                format!("$-1\r\n")
            }
        }
    }

    fn check_arity(cmd: String, arguments_len: usize) -> Option<Reply> {
        let (low, high) = match cmd.to_uppercase().as_str() {
            "PING" => (0, 1),
            "ECHO" => (1, 1),
            "SET" => (2, 2),
            "GET" => (1, 1),
            _ => (0, 100),
        };
        if arguments_len < low || arguments_len > high {
            Some(Reply::Error(CommandError::WrongNumberOfArguments(
                cmd.clone(),
            )))
        } else {
            //None bedeutet wir haben keinen Error
            None
        }
    }

    fn from_command(args: &[String], store: &mut HashMap<String, String>) -> Reply {
        let (cmd, args) = match args.split_first() {
            Some((cmd, args)) => (cmd, args),
            None => {
                return Reply::Error(CommandError::WrongNumberOfArguments(String::from(
                    "Splitting",
                )))
            }
        };

        if let Some(arrity_error) = Reply::check_arity(cmd.to_string(), args.len()) {
            return arrity_error;
        }
        match cmd.to_uppercase().as_str() {
            "PING" => {
                if let Some(argument) = args.first() {
                    Reply::BulkString(argument.clone())
                } else {
                    Reply::Pong
                }
            }
            "ECHO" => {
                let argument = args
                    .first()
                    .expect("Argument is null, even though we arity checked");
                Reply::BulkString(argument.clone())
            }
            "COMMAND" => {
                let argument = args
                    .first()
                    .expect("Argument is null, even though we arity checked");
                if argument == "DOCS" {
                    Reply::SimpleString("OK".to_string())
                } else {
                    Reply::Error(CommandError::UnknownCommand(cmd.clone()))
                }
            }
            "SET" => {
                store.insert(
                    args.first().unwrap().to_string(),
                    args.last().unwrap().to_string(),
                );
                Reply::SimpleString("OK".to_string())
            }
            "GET" => {
                //Wenn get etwas kriegt gib die Value aus sonst NULL
                let argument = args
                    .first()
                    .expect("Argument is null, even though we arity Checked");
                match store.get(argument) {
                    Some(value) => Reply::BulkString(value.clone()),
                    None => Reply::NullBulkString("Not Found".to_string()),
                }
            }
            _ => Reply::Error(CommandError::UnknownCommand(cmd.clone())),
        }
    }
}

fn handle_command(args: &[String], store: &mut HashMap<String, String>) -> String {
    //Hier später BusisnessLogic anhand des Commands erstellen
    //Rückgabewert ist die der String der zur Ausgabe gemacht wird. (Reply)
    let reply = Reply::from_command(args, store);
    reply.encode()
}

fn main() {
    let mut store: HashMap<String, String> = HashMap::new();
    let stdin = io::stdin();
    let mut input = stdin.lock();
    let mut out = io::stdout();

    loop {
        let mut header: String = String::new();
        if input.read_line(&mut header).unwrap() == 0 {
            break;
        }
        let count: usize = header
            .trim()
            .strip_prefix("*")
            .expect("Header did not have * as a first Symbol")
            .parse()
            .expect("Couldn't parse Number of Arguments from Header");

        //eprintln!("Header: {}", header);
        //eprintln!("Count: {}", count);

        //Dieser Loop ist über die Anzahl EINES Arguments als ein Set xxx wir kriegen für jedes
        // Argument die Länge des Arguements also wieviele Zeilen und pro Zeile wie Lang die Nächste ist
        let mut args: Vec<String> = Vec::with_capacity(count);
        for _i in 0..count {
            let mut line = String::new();
            let _ = input.read_line(&mut line).unwrap();
            //eprintln!("Line: {}", line);
            let len: usize = line.trim().strip_prefix("$").unwrap().parse().unwrap();
            //Buffer kriegt das komplette Argument mit \r\n am Ende
            // Es ist wirklich ein Argument als *3 würde sagen 1 SET 2 FOO 3 BAR als Argument danach geht der Loop von vorne los
            let mut buf = vec![0u8; len + 2]; //\rn ist 1 byte nicht 2
            let _ = input.read_exact(&mut buf);
            buf.truncate(len); // Wegschneiden \r\n
            let argument = String::from_utf8(buf).unwrap();
            //eprintln!("Argument: {:?}", argument);

            //Kann als String eingebaut werden
            args.push(argument);
        }
        //args ist auch immer zuständig für ein komplettes Argument nicht alle e.g
        // SET FOO BAR nicht SET FOO BAR GET FOO
        //
        //eprintln!("args: {:?}", args);
        out.write_all(handle_command(&args, &mut store).as_bytes())
            .unwrap();
        let _ = out.flush();
    }
}
