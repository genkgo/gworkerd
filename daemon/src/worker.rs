#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct Request {
	id: String,
	command: String,
	cwd: String,
}

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct Response {
	status: String,
	stderr: String,
	stdout: String
}

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct Item {
	request: Request,
	response: Response
}