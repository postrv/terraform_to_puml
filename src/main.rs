mod terraform_parser;

use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use serde::{Deserialize, Serialize};
use tera::Tera;
use async_openai::{Client as OpenAIClient, Client};
use async_openai::types::{
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
    ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
};
use ruml::{render_plantuml, Entity};
use crate::terraform_parser::parse_terraform;
use log::info;
use env_logger;
use actix_files::NamedFile;
use std::path::PathBuf;
use async_openai::config::OpenAIConfig;
use dotenv::dotenv;

#[derive(Deserialize)]
struct TerraformCode {
    code: String,
}

#[derive(Serialize)]
struct PumlResponse {
    puml: String,
}

async fn terraform_to_puml(data: web::Json<TerraformCode>) -> impl Responder {
    info!("Received Terraform to PUML request");
    let parsed_entities = parse_terraform(&data.code);
    let puml_diagram = render_plantuml(parsed_entities);
    HttpResponse::Ok().json(PumlResponse { puml: puml_diagram })
}

#[derive(Deserialize)]
struct AIRequest {
    description: String,
    current_code: String,
}

#[derive(Serialize)]
struct AIResponse {
    updated_code: String,
    puml: String,
}

async fn index() -> actix_web::Result<NamedFile> {
    let path: PathBuf = "./index.html".parse().unwrap();
    Ok(NamedFile::open(path)?)
}

async fn ai_assist_edit(data: web::Json<AIRequest>) -> impl Responder {
    info!("Received AI-assisted edit request");
    dotenv().ok();
    let openai_config = OpenAIConfig::new()
        .with_api_key(dotenv::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set"))
        .with_api_base("https://api.openai.com/v1");

    let openai_client = Client::with_config(openai_config);

    let prompt = format!(
        "Given the following Terraform code:\n{}\n\nMake the following changes:\n{}",
        data.current_code, data.description
    );

    let system_message = ChatCompletionRequestSystemMessageArgs::default()
        .content("You are an AI assistant that helps with editing Terraform code.")
        .build()
        .unwrap();

    let user_message = ChatCompletionRequestUserMessageArgs::default()
        .content(format!(
            "Given the following Terraform code:\n{}\n\nMake the following changes:\n{}",
            data.current_code, data.description
        ))
        .build()
        .unwrap();

    let messages = vec![
        ChatCompletionRequestMessage::from(system_message),
        ChatCompletionRequestMessage::from(user_message),
    ];

    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-3.5-turbo")
        .messages(messages)
        .build()
        .unwrap();

    let response = openai_client.chat().create(request).await.unwrap();
    let ai_response = response.choices[0].message.content.clone().unwrap();

    // Parse the AI-generated Terraform code
    let parsed_entities = parse_terraform(&ai_response);

    // Generate the updated PUML diagram
    let puml_diagram = render_plantuml(parsed_entities);

    HttpResponse::Ok().json(AIResponse { updated_code: ai_response, puml: puml_diagram })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    info!("Starting server at http://127.0.0.1:8080");

    let tera = Tera::new("templates/**/*").unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tera.clone()))
            .route("/", web::get().to(index))
            .route("/convert", web::post().to(terraform_to_puml))
            .route("/ai-edit", web::post().to(ai_assist_edit))
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
