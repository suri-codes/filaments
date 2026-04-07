use dto::{DatabaseConnection, TagEntity, ZettelEntity};
use nucleo_matcher::{
    Matcher, Utf32Str,
    pattern::{CaseMatching, Normalization, Pattern},
};
use tower_lsp::{
    Client, LanguageServer,
    jsonrpc::{self, Result},
    lsp_types::{
        CompletionItem, CompletionOptions, CompletionParams, CompletionResponse, InitializeParams,
        InitializeResult, InitializedParams, MessageType, ServerCapabilities,
        TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions, Url,
    },
};

use crate::types::{Zettel, ZettelId};

#[derive(Debug)]
pub struct Backend {
    client: Client,
    db: DatabaseConnection,
}

impl Backend {
    pub const fn new(client: Client, db: DatabaseConnection) -> Self {
        Self { client, db }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::FULL),
                        ..Default::default()
                    },
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec!["[".to_string()]),
                    ..Default::default()
                }),
                ..ServerCapabilities::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let file_path = params
            .text_document_position
            .text_document
            .uri
            .to_file_path()
            .expect("The uri should be in file format.");

        eprintln!(
            "Hello, processing completion in file: {}",
            file_path.display()
        );

        let Ok(file) = tokio::fs::read_to_string(&file_path).await else {
            return Ok(None);
        };
        eprintln!("here are the file contents: {file}");

        let Ok::<ZettelId, color_eyre::eyre::Error>(src_zid) = file_path.try_into() else {
            return Ok(None);
        };

        let position = params.text_document_position.position;

        eprintln!("here is the position we are given: {position:#?}");

        // for some weird reason we have to add +1 here??
        let line = file.lines().nth(position.line as usize + 1).unwrap_or("");

        eprintln!("params: {params:#?}");

        // this means they are looking for a link completion
        if let Some(context) = params.context
            && Some("[") == context.trigger_character.as_deref()
        {
            eprintln!("we are looking for link stuff");
            // let mut matcher = Matcher::default();
            // // the query is going to be anything between the '[' character and the last known cursor position

            // let query = {
            //     let Some(start) = line.rfind('[') else {
            //         eprintln!(
            //             "could not find the start of the link thingy: here is the line: {line}"
            //         );
            //         return Ok(None);
            //     };

            //     // .expect("it has to exist in the line for this completion request to be fired");
            //     let end = position.character as usize;

            //     Pattern::parse(
            //         &line[start..=end],
            //         CaseMatching::Ignore,
            //         Normalization::Smart,
            //     )
            // };

            let responses = ZettelEntity::load()
                .with(TagEntity)
                .all(&self.db)
                .await
                .map_err(|_| jsonrpc::Error::internal_error())?
                .into_iter()
                .map(Into::<Zettel>::into)
                .filter_map(|z| {
                    // let mut buf = Vec::new();

                    // let score = query
                    //     .score(Utf32Str::new(&z.title, &mut buf), &mut matcher)
                    //     .unwrap_or_default();

                    // if score > 0 {
                    Some(CompletionItem::new_simple(
                        z.title,
                        "The title of the thing".to_owned(),
                    ))
                    // } else {
                    // None
                    // }
                })
                .collect();

            return Ok(Some(CompletionResponse::Array(responses)));
        }

        return Ok(None);

        // // multi threaded zig
        // // async rust with tokio

        // // this is for what completion again?
        // Ok(Some(CompletionResponse::Array(vec![
        //     CompletionItem::new_simple("Hello".to_string(), "Some detail".to_string()),
        //     CompletionItem::new_simple("Bye".to_string(), "More detail".to_string()),
        // ])))
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}
