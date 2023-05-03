use actix_multipart::{
    form::{
        bytes::Bytes as MpBytes, text::Text, DuplicateField, FieldGroupReader, Limits as MpLimits,
        MultipartCollect, MultipartForm, State as MpState,
    },
    Field as MpField, MultipartError as MpError,
};
use actix_web::{
    get,
    http::header::ContentType,
    post,
    web::{self, Bytes},
    App, HttpRequest, HttpResponse, HttpServer, Result as AwResult,
};
use futures::Future;
use maud::{html, Markup};
use moka::future::Cache;
use std::pin::Pin;
use tokio::sync::RwLock;

mod app_config;
mod consts;
mod convert;
mod file_io;
mod template;
mod util;

use app_config::*;
use consts::{IMG_DATA_FIELD_NAME, IMG_NYM_FIELD_NAME, MAX_CACHE_SIZE, MAX_RESOURCE_NAME_LENGTH};
use file_io::FileIo;
use template::client_error;
use util::{hash, validate_input};

pub struct AppState {
    file_io: RwLock<FileIo>,
}

#[get("/upload")]
async fn get_upload() -> AwResult<Markup> {
    Ok(html! {
        form method="POST" enctype="multipart/form-data" action="/upload" {
            label for=(IMG_NYM_FIELD_NAME) {
                "Entry name:"
            }
            input type="text" id=(IMG_NYM_FIELD_NAME) name=(IMG_NYM_FIELD_NAME)
            br;
            label for=(IMG_DATA_FIELD_NAME) {
                "Select an image to upload:"
            }
            input type="file" id=(IMG_DATA_FIELD_NAME) name=(IMG_DATA_FIELD_NAME) {}
            br;
            input type="submit" value="Submit" {}
        }
    })
}

struct UploadForm {
    image_name: Text<String>,
    image_data: MpBytes,
}

impl MultipartCollect for UploadForm {
    fn limit(field_name: &str) -> ::std::option::Option<usize> {
        match field_name {
            IMG_NYM_FIELD_NAME => Some(MAX_RESOURCE_NAME_LENGTH),
            IMG_DATA_FIELD_NAME => Some(get_config::max_file_size::<usize>() + 1024),
            _ => None,
        }
    }
    fn handle_field<'t>(
        req: &'t HttpRequest,
        field: MpField,
        limits: &'t mut MpLimits,
        state: &'t mut MpState,
    ) -> Pin<Box<dyn Future<Output = Result<(), MpError>> + 't>> {
        match field.name() {
            IMG_NYM_FIELD_NAME => Box::pin(<Text<String> as FieldGroupReader>::handle_field(
                req,
                field,
                limits,
                state,
                DuplicateField::Deny,
            )),
            IMG_DATA_FIELD_NAME => Box::pin(<MpBytes as FieldGroupReader>::handle_field(
                req,
                field,
                limits,
                state,
                DuplicateField::Deny,
            )),
            _ => Box::pin(std::future::ready(Result::Err(MpError::UnsupportedField(
                field.name().to_string(),
            )))),
        }
    }
    fn from_state(mut state: MpState) -> Result<Self, MpError> {
        Ok(Self {
            image_name: <Text<String> as FieldGroupReader>::from_state("image_name", &mut state)?,
            image_data: <MpBytes as FieldGroupReader>::from_state("image_data", &mut state)?,
        })
    }
}

#[post("/upload")]
async fn post_upload(
    mut payload: MultipartForm<UploadForm>,
    req: HttpRequest,
) -> AwResult<HttpResponse> {
    let content_length = req
        .headers()
        .get("Content-length")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0);
    if content_length > get_config::max_file_size::<usize>() + 1024 {
        return Ok(client_error::payload_too_large());
    }

    let key = if let Some(s) = validate_input(payload.0.image_name.as_str()) {
        hash(s)
    } else {
        return Ok(client_error::invalid_image_name());
    };

    Ok(HttpResponse::Ok().finish())
}

#[get("/i/{image_name}")]
async fn image(data: web::Data<AppState>, path: web::Path<String>) -> HttpResponse {
    let key = if let Some(s) = validate_input(&path) {
        hash(s)
    } else {
        return client_error::invalid_image_name();
    };
    let mut file_io = data.file_io.write().await;
    if let Ok(binary_data) = file_io.load_image(key).await {
        let mut response = HttpResponse::Ok();
        response.content_type(ContentType::jpeg());
        response.body(binary_data)
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = web::Data::new(AppState {
        file_io: RwLock::new(FileIo {
            cache: Cache::builder()
                .weigher(|_key, value: &Bytes| -> u32 {
                    value.len().try_into().unwrap_or(u32::MAX)
                })
                .max_capacity(MAX_CACHE_SIZE as u64)
                .build(),
        }),
    });
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(image)
            .service(get_upload)
            .service(post_upload)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
