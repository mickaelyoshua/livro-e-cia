use rocket::http::{Cookie, CookieJar, SameSite};
use time::Duration;

const ACCESS_COOKIE: &str = "access_token";
const REFRESH_COOKIE: &str = "refresh_token";

pub fn set_auth_cookies(
    cookies: &CookieJar<'_>,
    access_token: &str,
    refresh_token: &str,
    secure: bool, // On development it is not required https
) {
    let access = Cookie::build((ACCESS_COOKIE, access_token.to_string()))
        .path("/")
        .http_only(true)
        .secure(secure)
        .same_site(SameSite::Lax)
        .max_age(Duration::minutes(15));

    let refresh = Cookie::build((REFRESH_COOKIE, refresh_token.to_string()))
        .path("/auth")
        .http_only(true)
        .secure(secure)
        .same_site(SameSite::Lax)
        .max_age(Duration::days(7));

    // Encrypt the cookie value
    cookies.add_private(access);
    cookies.add_private(refresh);
}

pub fn remove_auth_cookies(cookies: &CookieJar<'_>) {
    cookies.remove_private(Cookie::build(ACCESS_COOKIE).path("/"));
    cookies.remove_private(Cookie::build(REFRESH_COOKIE).path("/auth"));
}

pub fn get_access_token(cookies: &CookieJar<'_>) -> Option<String> {
    cookies
        .get_private(ACCESS_COOKIE)
        .map(|c| c.value().to_string())
}

pub fn get_refresh_token(cookies: &CookieJar<'_>) -> Option<String> {
    cookies
        .get_private(REFRESH_COOKIE)
        .map(|c| c.value().to_string())
}
