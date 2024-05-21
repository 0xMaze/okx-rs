use crate::okx::Okx;

use reqwest::Url;

#[test]
fn test_url_build_with_params() {
    let params = vec![
        ("some".to_string(), "param".to_string()),
        ("another".to_string(), "params".to_string()),
    ];
    let url = Okx::build_url("/api/v5/account/balance", Some(params));
    let target_url = "https://www.okx.com/api/v5/account/balance?some=param&another=params"
        .parse::<Url>()
        .unwrap();
    assert_eq!(url, target_url)
}

#[test]
fn test_url_build_no_params() {
    let url = Okx::build_url("/api/v5/account/balance", None);
    println!("{url}");

    let target_url = "https://www.okx.com/api/v5/account/balance"
        .parse::<Url>()
        .unwrap();

    assert_eq!(url, target_url)
}
