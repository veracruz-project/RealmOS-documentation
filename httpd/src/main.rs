/// Example is derived from tiny-http example
/// https://github.com/tiny-http/tiny-http/blob/master/examples/hello-world.rs

#[cfg(target_os = "hermit")]
use hermit as _;

use tiny_http::{Server, Response};

fn main() {
	println!("Starting server on port 8080");
	let server = Server::http("0.0.0.0:8080").unwrap();
	println!("Now listening on port 8080");

	for request in server.incoming_requests() {
		println!("received request! method: {:?}, url: {:?}, headers: {:?}",
			request.method(),
			request.url(),
			request.headers()
		);
		let response = Response::from_string("hello world\n");
		request.respond(response).unwrap();
	}
}
