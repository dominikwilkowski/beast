use axum::serve;
use cfonts::{Align, Fonts, Options};
use std::{env, net::SocketAddr, sync::Arc};

mod errors;
mod server;
mod store;

use crate::server::HighscoreServer;

#[tokio::main]
async fn main() {
	let port = env::var("PORT").ok().and_then(|p| p.parse::<u16>().ok()).unwrap_or(8000);
	let address = SocketAddr::from(([127, 0, 0, 1], port));
	let db_path = env::var("DB_PATH").unwrap_or(String::from("highscores.ron"));
	let server = Arc::new(HighscoreServer::new(db_path));

	println!("\n");
	cfonts::say(Options {
		text: String::from("beast highscore"),
		font: Fonts::FontChrome,
		align: Align::Center,
		spaceless: true,
		gradient: vec![String::from("#ff0000"), String::from("#0000ff")],
		..Options::default()
	});
	cfonts::say(Options {
		text: format!("server running on {address}"),
		font: Fonts::FontConsole,
		align: Align::Center,
		spaceless: true,
		..Options::default()
	});

	let listener = match tokio::net::TcpListener::bind(address).await {
		Ok(listener) => listener,
		Err(error) => {
			eprintln!("Failed to bind to address: {error}");
			std::process::exit(1);
		},
	};

	match serve(listener, server.router().into_make_service()).await {
		Ok(_) => (),
		Err(error) => {
			eprintln!("Failed to serve: {error}");
			std::process::exit(1);
		},
	};
}
