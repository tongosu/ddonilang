use ddonirang_tool::lsp::{encode_message, FrameDecoder, LspServer};
use std::io::{self, Read, Write};

const READ_BUFFER_SIZE: usize = 8 * 1024;

fn run() -> Result<i32, String> {
    let mut decoder = FrameDecoder::default();
    let mut server = LspServer::new();
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    let mut input = [0_u8; READ_BUFFER_SIZE];
    loop {
        let read = stdin
            .read(&mut input)
            .map_err(|error| format!("stdin read failed: {error}"))?;
        if read == 0 {
            decoder.finish().map_err(|error| error.to_string())?;
            return Ok(0);
        }

        let messages = decoder
            .push(&input[..read])
            .map_err(|error| error.to_string())?;
        for message in messages {
            let event = server
                .handle_message(message)
                .map_err(|error| error.to_string())?;
            for response in event.responses {
                let frame = encode_message(&response).map_err(|error| error.to_string())?;
                stdout
                    .write_all(&frame)
                    .map_err(|error| format!("stdout write failed: {error}"))?;
                stdout
                    .flush()
                    .map_err(|error| format!("stdout flush failed: {error}"))?;
            }
            if let Some(status) = event.exit_status {
                return Ok(status);
            }
        }
    }
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
