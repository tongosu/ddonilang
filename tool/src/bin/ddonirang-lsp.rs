use ddonirang_tool::lsp::{encode_message, FrameDecoder, LspServer};
use std::io::{self, Read, Write};

fn run() -> Result<i32, String> {
    let mut input = Vec::new();
    io::stdin()
        .read_to_end(&mut input)
        .map_err(|error| format!("stdin read failed: {error}"))?;

    let mut decoder = FrameDecoder::default();
    let messages = decoder.push(&input).map_err(|error| error.to_string())?;
    decoder.finish().map_err(|error| error.to_string())?;

    let mut server = LspServer::new();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    for message in messages {
        let event = server
            .handle_message(message)
            .map_err(|error| error.to_string())?;
        for response in event.responses {
            let frame = encode_message(&response).map_err(|error| error.to_string())?;
            stdout
                .write_all(&frame)
                .map_err(|error| format!("stdout write failed: {error}"))?;
        }
        stdout
            .flush()
            .map_err(|error| format!("stdout flush failed: {error}"))?;
        if let Some(status) = event.exit_status {
            return Ok(status);
        }
    }
    Ok(0)
}

fn main() {
    match run() {
        Ok(status) => std::process::exit(status),
        Err(error) => {
            eprintln!("ddonirang-lsp: {error}");
            std::process::exit(1);
        }
    }
}
