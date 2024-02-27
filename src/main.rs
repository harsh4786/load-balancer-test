use std::{io::{self, Read, Write}, net::{TcpListener, TcpStream, ToSocketAddrs}, thread};


fn main() {
    let listen_addr = "127.0.0.1:8080";
    let backends = vec![
        "127.0.0.1:8081".to_string(),
        "127.0.0.1:8082".to_string(),
    ];

    start_load_balancer(listen_addr, backends)
        .expect("Failed to start the load balancer");
}


struct BackendPool {
    backends: Vec<String>,
    current: usize,
}

impl BackendPool {
    fn new(backends: Vec<String>) -> Self {
        BackendPool { backends, current: 0 }
    }

    fn get_next_backend(&mut self) -> &String {
        let backend = &self.backends[self.current];
        self.current = (self.current + 1) % self.backends.len();
        backend
    }
}


fn handle_client(mut stream: TcpStream, backend_addr: String) -> io::Result<()> {
    let mut backend_stream = TcpStream::connect(backend_addr)?;
    let mut buffer = [0; 4096];

    // Simple forward and response (proxy-like behavior)
    loop {
        let bytes_read = stream.read(&mut buffer)?;
        if bytes_read == 0 { break; }

        backend_stream.write_all(&buffer[..bytes_read])?;
        let bytes_written = backend_stream.read(&mut buffer)?;
        stream.write_all(&buffer[..bytes_written])?;
    }

    Ok(())
}

fn start_load_balancer<A: ToSocketAddrs>(listen_addr: A, backends: Vec<String>) -> io::Result<()> {
    let listener = TcpListener::bind(listen_addr)?;
    let mut backend_pool = BackendPool::new(backends);

    // Accept connections and process them.
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let backend_addr = backend_pool.get_next_backend().clone();
                thread::spawn(move || {
                    handle_client(stream, backend_addr).unwrap_or_else(|error| eprintln!("{:?}", error));
                });
            }
            Err(e) => { eprintln!("Failed to accept connection: {}", e); }
        }
    }

    Ok(())
}

