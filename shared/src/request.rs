
pub async fn request<F>(f: F) -> Result<String, reqwest::Error>
where
    F: Fn(reqwest::Client) -> reqwest::RequestBuilder,
{
    f(reqwest::Client::new()).send().await?.text().await
}
