use std::io::{self, BufRead, Read, Write};
enum Reply {
    Pong,
    SimpleString(String),
    BulkString(String),
    Error(CommandError),
    //Number can also be float...
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
    // Hier wäre Gedanke, dass wenn ein Error kommt, geben wir den zurück, sonst geben wir einfach
    // nichts zurück.
    fn check_arity(cmd: String, arguments_len: usize) -> Option<Reply> {
        let (low, high) = match cmd.to_uppercase().as_str() {
            "PING" => (0, 1),
            "ECHO" => (1, 1),
            _ => (0, 100),
        };
        if arguments_len < low || arguments_len > high {
            Some(Reply::Error(CommandError::WrongNumberOfArguments(
                cmd.clone(),
            )))
        } else {
            None
        }
        //if Beidnugen => Some else NONE none bedeutet keine fehler und wir gehen weiter.
        //Arity is the number of arguments or operands that a function, operation, or relation takes in logic, mathematics, and computer science
    }
    fn from_command(args: &[String]) -> Reply {
        //let Some(x,y) gibt mr wenn möglich beide Variablen sonst error ...
        //Dadurch habe ich jetzt cmd und arguments was ist arguments?
        //Tuple
        //Syntactic Sugar
        //let Some((cmd, arguments)) = args.split_first() else {
        //    return Reply::Error("missing cmd".to_string());
        //};

        //Funktioniert nicht auf den Dingern daher richtig match
        //Genau dasselbe nur mehr verbos

        let (cmd, arguments) = match args.split_first() {
            Some((cmd, arguments)) => (cmd, arguments),
            None => {
                return Reply::Error(CommandError::WrongNumberOfArguments(String::from(
                    "Splitting",
                )))
            }
        };

        if let Some(arrity_error) = Reply::check_arity(cmd.to_string(), arguments.len()) {
            return arrity_error;
        }

        //println!("STUFF: \n {}", cmd.to_uppercase().as_str());
        match cmd.to_uppercase().as_str() {
            "PING" => {
                //Wenn ein Argument existiert, dann packe es in die Variable und arbeite weiter?
                //Some ist Optionals in Java nur hole sie mir mit Some raus. und gibt mirkeinen
                //Error dann umgeschireben if Some(argument) = argument.first {
                //} stattdessen so
                if let Some(argument) = arguments.first() {
                    Reply::BulkString(argument.clone())
                } else {
                    Reply::Pong
                }
            }
            "ECHO" => {
                //Wir benutzen hier expect, weil wir schon durch arity check
                //geguckt haben, dass etwas in first() sitzt als Parameter
                //wenn es durchgeht dann sollten wir panicen
                //keine Ifs mehr weil arity check.
                let argument = arguments
                    .first()
                    .expect("Argument is null, even though we arity checked");
                Reply::BulkString(argument.clone())
            }
            "COMMAND" => {
                let argument = arguments
                    .first()
                    .expect("Argument is null, even though we arity checked");
                if argument == "DOCS" {
                    Reply::SimpleString("OK".to_string())
                } else {
                    Reply::Error(CommandError::UnknownCommand(cmd.clone()))
                }
            }
            _ => Reply::Error(CommandError::UnknownCommand(cmd.clone())),
        }
    }

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

    fn parse_args(line: &str) -> Vec<String> {
        let mut args = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        for ch in line.chars() {
            match ch {
                '"' if !in_quotes => in_quotes = true,
                '"' if in_quotes => in_quotes = false,
                ' ' if !in_quotes => {
                    if !current.is_empty() {
                        args.push(current.clone());
                        current.clear();
                    }
                }
                _ => current.push(ch),
            }
        }
        if !current.is_empty() {
            args.push(current);
        }
        args
    }
}

fn encode_bulk_string(s: &str) -> String {
    format!("${}\r\n{}\r\n", s.len(), s)
}

fn handle_command(args: &[String]) -> String {
    let reply = Reply::from_command(args);
    reply.encode()
}

fn main() {
    let stdin = io::stdin();
    let mut input = stdin.lock();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    //let mut header: Vec<String> = Vec::new();
    //let mut message_header = Vec::new();
    //let mut input_buffer = Vec::new();

    //stdin.lock().read_to_end(&mut input_buffer).unwrap();
    //
    //
    //

    //First Line is always the amount of messages, we can just get those of readline.

    loop {
        let mut header: String = String::new();

        if input.read_line(&mut header).unwrap() == 0 {
            break; //done
        }

        let count = header.trim().strip_prefix("*").unwrap().parse().unwrap();
        //This always has the Form of *<NUM> so cut off the first Symbol and then parse the number
        //TODO better way to not use double unwrap...

        let mut args = Vec::with_capacity(count);

        //We can always read the amount of incoming bytes with a line
        for i in 0..count {
            let mut line = String::new();
            let _ = input.read_line(&mut line).unwrap();
            let len: usize = line.trim().strip_prefix("$").unwrap().parse().unwrap();
            let mut buf = vec![0u8; len + 2]; // get last 2 bytes \r\n
            let _ = input.read_exact(&mut buf);
            buf.truncate(len);

            //add the actual Strings to our arguments like before
            args.push(String::from_utf8(buf).unwrap());
        }
        eprintln!("args: {:?}", args);
        out.write_all(handle_command(&args).as_bytes()).unwrap();
        let _ = out.flush();
    }
}
