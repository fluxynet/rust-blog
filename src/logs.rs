use hostname;
use tracing_loki::BackgroundTask;
use tracing_subscriber::layer::SubscriberExt;
use url::Url;

pub fn loki(url: String) -> BackgroundTask {
    let loki_url = Url::parse(format!("{}/loki/api/v1/push", url).as_str()).unwrap();
    let host = hostname::get().unwrap().to_str().unwrap().to_string();

    let (layer, task) = tracing_loki::builder()
        .label("host", host)
        .unwrap()
        .build_url(loki_url)
        .unwrap();

    tracing_subscriber::registry()
        .with(layer)
        .with(tracing_subscriber::fmt::Layer::new());

    task
}
