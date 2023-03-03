use super::model::in_memory_index_model::Model;
use std::io;
use tiny_http::{Header, Method, Request, Response, Server, StatusCode};

pub struct WebServer<'a> {
    pub addr: &'a str,
    pub model: Box<dyn Model>,
}

impl<'a> WebServer<'a> {
    pub fn new(addr: &'a str, model: Box<dyn Model>) -> Self {
        WebServer { addr, model }
    }

    fn serve_404(request: Request) -> std::io::Result<()> {
        request.respond(Response::from_string("404").with_status_code(StatusCode(404)))
    }

    fn serve_500(request: Request) -> std::io::Result<()> {
        request.respond(Response::from_string("500").with_status_code(StatusCode(500)))
    }

    fn serve_static_file(
        request: Request,
        file_path: &str,
        content_type: &str,
    ) -> std::io::Result<()> {
        let content_type_header = Header::from_bytes("Content-Type", content_type)
            .expect("Response header should not be empty.");

        let file = match std::fs::File::open(file_path) {
            Ok(file) => file,
            Err(err) => {
                eprintln!("ERROR: could not serve the file {file_path}: {err}");
                if err.kind() == io::ErrorKind::NotFound {
                    return Self::serve_404(request);
                }
                return Self::serve_500(request);
            }
        };

        let response = Response::from_file(file).with_header(content_type_header);
        request.respond(response)?;

        Ok(())
    }

    fn serve_search(&self, mut request: Request) -> std::io::Result<()> {
        let mut query = String::new();
        if let Err(err) = request.as_reader().read_to_string(&mut query) {
            eprintln!("ERROR: could not read the body of the request: {err}");
            return Self::serve_500(request);
        };

        println!("Request body(query): {query}");

        let results = match self.model.search(&query.chars().collect::<Vec<char>>()) {
            Ok(result) => result,
            Err(_) => return Self::serve_500(request),
        };

        let mut data = Vec::<(String, f32)>::new();
        for (path, rank) in results.iter().take(15) {
            println!("File Path: {path} | Rank: {rank}", path = path.display());

            data.push((format!("{path}", path = path.display()), *rank));
        }

        let json = match serde_json::to_string(&data) {
            Ok(json) => json,
            Err(err) => {
                eprintln!("ERROR: could not convert search results to JSON, search result: {data:?}: {err}");
                return Self::serve_500(request);
            }
        };

        let header = Header::from_bytes("Content-Type", "application/json")
            .expect("Response header should not be empty.");

        let response = Response::from_string(json)
            .with_header(header)
            .with_status_code(tiny_http::StatusCode(200));

        request.respond(response)?;

        Ok(())
    }

    fn serve_request(&self, request: Request) -> io::Result<()> {
        println!(
            "INFO: received request! method: {method}, url: {url}",
            method = request.method(),
            url = request.url()
        );

        match (request.method(), request.url()) {
            (Method::Get, "/") => {
                Self::serve_static_file(request, "index.html", "text/html; charset=utf-8")?;
            }
            (Method::Get, "/index.js") => {
                Self::serve_static_file(request, "index.js", "text/javascript; charset=utf-8")?;
            }
            (Method::Post, "/api/search") => {
                self.serve_search(request)?;
            }
            _ => {
                println!("Other url: {url}", url = request.url());
                return Self::serve_404(request);
            }
        }
        Ok(())
    }

    pub fn start(&self) -> Result<(), ()> {
        let server = Server::http(self.addr).map_err(|err| {
            eprintln!(
                "ERROR: could not build up the server at {addr}: {err}",
                addr = self.addr
            );
        })?;

        println!("INFO: listening at http://{addr}/", addr = self.addr);

        for request in server.incoming_requests() {
            println!(
                "method: {method:#?}, url: {url:#?}",
                method = request.method(),
                url = request.url(),
            );

            self.serve_request(request)
                .map_err(|err| eprintln!("ERROR: could not serve the response: {err}"))
                .ok(); // do not stop on errors, keep going
        }

        eprintln!("ERROR: the server has shutdown");
        Err(())
    }
}
