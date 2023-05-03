pub mod client_error {
    use actix_web::HttpResponse;
    use maud::html;

    pub fn payload_too_large() -> HttpResponse {
        HttpResponse::BadRequest().body(
            html! {
                "Payload too large."
            }
            .into_string(),
        )
    }

    pub fn invalid_image_name() -> HttpResponse {
        HttpResponse::BadRequest().body(
            html! {
                "Invalid image name."
            }
            .into_string(),
        )
    }
}
