use axum::{routing::post, serve::Serve, Router};
use std::error::Error;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

pub mod routes;

pub struct Application {
    server: Serve<Router, Router>,
    pub address: String,
}

impl Application {
    pub async fn build(address: &str) -> Result<Self, Box<dyn Error>> {
        let router = Router::new()
            .nest_service("/", ServeDir::new("assets"))
            .route("/signup", post(routes::post_signup))
            .route("/login", post(routes::post_login))
            .route("/logout", post(routes::post_logout))
            .route("/verify-2fa", post(routes::post_verify_2fa))
            .route("/verify-token", post(routes::post_verify_token));

        let listener = TcpListener::bind(address).await?;
        let address = listener.local_addr()?.to_string();
        let server = axum::serve(listener, router);

        let app = Application { server, address };

        Ok(app)
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        println!("listening on {}", &self.address);
        self.server.await
    }
}
