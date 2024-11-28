#![allow(unreachable_code)]
#![allow(dead_code)]
#![allow(clippy::diverging_sub_expression)]
#![allow(clippy::unreachable)]

mod cli;
mod database;
mod deus_ex_machina;
mod error;
mod model;
mod sql_queries;
mod state;
mod utils;

use std::{convert::Infallible, fs::remove_dir_all, net::ToSocketAddrs};

use async_graphql::{http::GraphiQLSource, SDLExportOptions, Schema};
use async_graphql_axum::{GraphQL, GraphQLSubscription};
use axum::{
    response::Html,
    routing::{get, post_service},
    Router,
};
use axum_helpers::{app::AxumApp, response_http_header_mutator::ResponseHttpHeaderMutatorLayer};
use clap::Parser;
use cli::{Cli, Commands, SchemaSource};
use deus_ex_machina::DeusExMachina;
use graphql_cli_tools::schema_diff::diff_schema;
use model::{mutation::Mutation, query::Query, subscription::Subscription};
use state::State;
use tower_http::trace::TraceLayer;

fn routes(state: State, schema: Schema<Query, Mutation, Subscription>) -> Router {
    let preflight_middleware = ResponseHttpHeaderMutatorLayer::new(|_req_headers, res_headers| {
        res_headers.insert(
            "Access-Control-Allow-Methods",
            "*".parse().expect("cannot parse HTTP header value"),
        );
        res_headers.insert(
            "Access-Control-Allow-Headers",
            "*".parse().expect("cannot parse HTTP header value"),
        );
        res_headers.insert(
            "Access-Control-Allow-Origin",
            "*".parse().expect("cannot parse HTTP header value"),
        );

        Ok::<(), Infallible>(())
    });

    Router::new()
        .route("/", get(index_page))
        .route("/graphiql", get(graphiql))
        .route_service("/api/graphql-ws", GraphQLSubscription::new(schema.clone()))
        .route(
            "/api/graphql",
            post_service(GraphQL::new(schema.clone()))
                .options(options_graphql)
                .route_layer(preflight_middleware.clone()),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn options_graphql() {}

async fn index_page() -> Html<String> {
    Html(
        r#"
            <html>
                <body>
                    <a href="/graphiql">GraphiQL</a>
                </body>
            </html>
        "#
        .into(),
    )
}

async fn graphiql() -> Html<String> {
    Html(
        GraphiQLSource::build()
            .endpoint("/api/graphql")
            .subscription_endpoint("/api/graphql-ws")
            .finish(),
    )
}

fn create_schema(state: State) -> Schema<Query, Mutation, Subscription> {
    let query = Query {
        state: state.clone(),
    };
    let mutation = Mutation {
        _state: state.clone(),
    };
    let subscription = Subscription {
        _state: state.clone(),
    };
    Schema::build(query, mutation, subscription)
        .extension(DeusExMachina::new(state))
        .finish()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::new()
        // .filter_level(log::LevelFilter::Debug)
        .filter_module("axum_helpers", log::LevelFilter::Debug)
        .filter_module("graphql-example", log::LevelFilter::Trace)
        .filter_module("tower_http", log::LevelFilter::Debug)
        .init();

    let cli = Cli::parse();

    let db_path = "db";

    if cli.purge_db {
        let _ = remove_dir_all(db_path);
    }

    let state = State::new(db_path).await?;
    let schema = create_schema(state.clone());

    match cli.command {
        Commands::Serve(params) => {
            log::info!("starting application in server mode");

            let mut app = AxumApp::new(routes(state, schema));
            for addr in params.listener_address.to_socket_addrs()? {
                let _ = app.spawn_server(addr).await.inspect_err(|e| {
                    log::error!(
                        "{}, could not listen on address = {addr}, error = {e:?}",
                        log_location!()
                    );
                });
            }

            app.join().await;
        }
        Commands::Sdl => {
            println!(
                "{}",
                schema.sdl_with_options(SDLExportOptions::new().prefer_single_line_descriptions())
            );
        }
        Commands::DiffSchema(params) => {
            match (params.schema_source_left, params.schema_source_right) {
                (SchemaSource::File(path_left), SchemaSource::File(path_right)) => {
                    diff_schema(path_left, path_right)?;
                }
                (SchemaSource::SelfSchema, SchemaSource::File(path_right)) => {
                    diff_schema(schema.sdl(), path_right)?;
                }
                (SchemaSource::File(path_left), SchemaSource::SelfSchema) => {
                    diff_schema(path_left, schema.sdl())?;
                }
                (SchemaSource::SelfSchema, SchemaSource::SelfSchema) => {
                    diff_schema(schema.sdl(), schema.sdl())?;
                }
            }
        }
    }

    Ok(())
}
