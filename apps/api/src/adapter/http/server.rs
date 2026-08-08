use axum::Router;
use tokio::net::TcpListener;

pub async fn bind(address: &str) -> Result<TcpListener, std::io::Error> {
    TcpListener::bind(address).await
}

pub async fn serve(listener: TcpListener, router: Router) -> Result<(), std::io::Error> {
    axum::serve(listener, router).await
}
