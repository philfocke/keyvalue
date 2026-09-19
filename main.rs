use std::io::{self, BufRead, Write};



enum Reply {
    Pong,
    SimpleString(String),
    BulkString(String),
    Error(String),
    //Number can also be float...
    Number(isize),
    NullBulkString(String),
}

impl Reply {
    fn from(arg: &String, arg_len: usize) -> Reply {
        let cmd = arg.to_uppercase();
        match cmd.as_str() {
            "PING" =>  {
                if arg_len > 1 {
                    Reply::SimpleString(String::clone(&arg))
                } else {
                    Reply::Pong
                }
            },
            _ => Reply::Error(String::from("not yet Implemented"))
        }

    }


    fn get_format_string(&self) -> String {
        match self {
            Reply::Pong => String::from("+PONG\r\n"),
            _ => String::from("Not yet Implemented"),
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
    if !current.is_empty() { args.push(current); }
    args
}

fn encode_bulk_string(s: &str) -> String {
    format!("${}\r\n{}\r\n", s.len(), s)
}

fn handle_command(args: &[String]) -> String {
    let cmd = args[0].to_uppercase();
    let reply = Reply::from(&cmd, args.len());
    reply.get_format_string()
    

    //len() gives bytes in utf8 not character length :)!
//    match cmd.as_str() {
//        "PING" => {
//            if args.len() > 1 {
//                format!("${}\r\n{}\r\n",args[1].len(), args[1])
//            } else {
//                String::from("+PONG\r\n")
//            }
//        }
//        "ECHO" => {
//            format!("${}\r\n{}\r\n",args[1].len(), args[1])
//        }
//        // TODO: Return "+PONG\r\n" for no args
//        // TODO: Return bulk string for PING <message>
//        _ => format!("-ERR unknown command\r\n"),
//    }
}

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let line = line.trim().to_string();
        if line.is_empty() { continue; }
        let args = parse_args(&line);
        let response = handle_command(&args);
        write!(out, "{}", response).unwrap();
        out.flush().unwrap();
    }
}
