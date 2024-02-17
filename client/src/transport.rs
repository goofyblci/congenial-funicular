use anyhow::Result;
use http_body_util::{BodyExt, Empty};
use hyper::body::Bytes;
use hyper::http::uri::Scheme;
use hyper::{Request, StatusCode, Uri};
use hyper_util::rt::TokioIo;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::time;
use tokio_native_tls::native_tls::TlsConnector;

use arti_client::{StreamPrefs, TorClient, TorClientConfig};

use crate::app::App;

pub async fn make_test_connection(app: &mut App) -> Result<()> {
    let url: Uri =
        "http://ucd2in7e4aiakufoafjj5uwy3in3neqdspknwrnyfhi7n73ow3b5zvid.onion".parse()?;
    let host = url.host().unwrap();
    let https = url.scheme() == Some(&Scheme::HTTPS);
    let mut config = TorClientConfig::builder();
    config
        .address_filter()
        .allow_onion_addrs(true)
        .build()
        .unwrap();
    // let stream_prefs = StreamPrefs::new();
    let final_config = config.build().unwrap();
    let client = TorClient::create_bootstrapped(final_config).await?;
    let port = match url.port_u16() {
        Some(port) => port,
        _ if https => 443,
        _ => 80,
    };
    let stream = client.connect((host, port)).await.unwrap();
    if https {
        let cx = TlsConnector::builder().build().unwrap();
        let cx = tokio_native_tls::TlsConnector::from(cx);
        let stream = cx.connect(host, stream).await.unwrap();
        app.set_tor_status_code(StatusCode::OK);
        make_request(host, stream, app).await
    } else {
        make_request(host, stream, app).await
    }
}

async fn make_request(
    host: &str,
    stream: impl AsyncRead + AsyncWrite + Unpin + Send + 'static,
    app: &mut App,
) -> Result<()> {
    let (mut request_sender, connection) =
        hyper::client::conn::http1::handshake(TokioIo::new(stream)).await?;

    // spawn a task to poll the connection and drive the HTTP state
    tokio::spawn(async move {
        connection.await.unwrap();
    });

    let mut resp = request_sender
        .send_request(
            Request::builder()
                .header("Host", host)
                .method("GET")
                .body(Empty::<Bytes>::new())?,
        )
        .await?;

    println!("status: {}", resp.status());

    app.set_tor_status_code(resp.status());

    let now = time::Instant::now();

    while let Some(frame) = resp.body_mut().frame().await {
        let bytes = frame?.into_data().unwrap();
        println!("body: {}", std::str::from_utf8(&bytes)?);
        let vec_bytes = bytes.to_vec();
        app.set_tor_response_body(vec_bytes);
        if now.elapsed().as_secs() > 10 {
            break;
        }
    }

    Ok(())
}
