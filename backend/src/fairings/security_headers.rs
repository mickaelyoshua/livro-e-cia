use rocket::{
    fairing::{Fairing, Info, Kind},
    http::Header,
    Request, Response,
};

pub struct SecurityHeaders;

#[rocket::async_trait]
impl Fairing for SecurityHeaders {
    fn info(&self) -> Info {
        Info {
            name: "Security Headers",
            kind: Kind::Response, // add headers to outgoing responses
        }
    }

    async fn on_response<'r>(&self, _req: &'r Request<'_>, res: &mut Response<'r>) {
        // Prevents MIME-type sniffing attacks by forcing browsers to respect declared content types
        res.set_header(Header::new("X-Content-Type-Options", "nosniff"));

        // Prevents clickjacking attacks by blocking this site from being embedded in iframes
        res.set_header(Header::new("X-Frame-Options", "DENY"));

        // Legacy XSS protection - blocks page load when reflected XSS is detected
        res.set_header(Header::new("X-XSS-Protection", "1; mode=block"));

        // Controls referrer information: sends full URL for same-origin, only origin for cross-origin
        res.set_header(Header::new(
            "Referrer-Policy",
            "strict-origin-when-cross-origin",
        ));

        // Prevents XSS and data injection attacks by restricting resource sources to same origin only
        res.set_header(Header::new(
            "Content-Security-Policy",
            "default-src 'self'; frame-ancestors 'none'",
        ));
    }
}
