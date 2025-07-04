pub(crate) mod args;
pub(crate) mod config;
pub(crate) mod openrouter;

use std::fs::read_to_string;

use actix_web::{
    App, HttpResponse, HttpResponseBuilder, HttpServer, Responder, get,
    http::StatusCode, web,
};
use anyhow::Context;
use clap::Parser;

use crate::openrouter::response;

const BASE_URL: &str = "https://openrouter.ai/api/v1/";

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let args: args::Args = args::Args::parse();
    let config: config::Config = toml::from_str(
        &read_to_string(args.config).context("Cannot read the config file")?,
    )
    .context("Invalid config file")?;

    HttpServer::new(|| App::new().service(models))
        .bind((config.server.address, config.server.port))?
        .run()
        .await?;

    Ok(())
}

#[get("/api/v1/models")]
async fn models(
    query: web::Query<openrouter::query::ListAvailableModels>,
) -> impl Responder {
    let client: reqwest::Client = reqwest::Client::new();

    let result: reqwest::Response = match client
        .get(format!("{BASE_URL}models/"))
        .query(&query.into_inner())
        .send()
        .await
    {
        Ok(result) => result,
        Err(err) => {
            return HttpResponse::InternalServerError().json(
                response::ErrorResponse {
                    error: response::ErrorResponseInner {
                        code: 500,
                        message: format!("{err:#?}"),
                        metadata: None,
                    },
                },
            );
        }
    };
    let status: StatusCode = StatusCode::from_u16(result.status().as_u16())
        .expect("Cannot convert the status code");

    let body: response::Response<response::ListAvailableModels> =
        match result.json().await {
            Ok(body) => body,
            Err(err) => {
                return HttpResponse::InternalServerError().json(
                    response::ErrorResponse {
                        error: response::ErrorResponseInner {
                            code: 500,
                            message: format!("{err:#?}"),
                            metadata: None,
                        },
                    },
                );
            }
        };
    HttpResponseBuilder::new(status).json(body)
}
