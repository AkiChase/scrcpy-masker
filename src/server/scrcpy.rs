use tokio::net::TcpStream;

pub struct ScrcpyConnection {
    pub socket: TcpStream,
}

impl ScrcpyConnection {
    pub fn new(socket: TcpStream) -> Self {
        ScrcpyConnection { socket }
    }

    pub async fn handle(&mut self) {
        // TODO handle scrcpy connection
        log::debug!("handle scrcpy connection...")
    }
}

pub fn start_scrcpy_server() {
    // TODO push server jar, shell_process
}
