use color_eyre::eyre::eyre;
use dto::{TagEntity, ZettelEntity};
use tower_lsp::{
    Client, LanguageServer,
    jsonrpc::{self, Result},
    lsp_types::{
        CompletionItem, CompletionOptions, CompletionParams, CompletionResponse, InitializeParams,
        InitializeResult, InitializedParams, MessageType, ServerCapabilities,
        TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions,
    },
};

use crate::types::{Kasten, Tag, Zettel, ZettelId};

#[derive(Debug)]
pub struct Backend {
    client: Client,
    kt: Kasten,
}

impl Backend {
    pub const fn new(client: Client, kt: Kasten) -> Self {
        Self { client, kt }
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
                    trigger_characters: Some(vec!["[[".to_string(), "@".to_string()]),
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
        let Some(trigger) = params
            .context
            .as_ref()
            .and_then(|c| c.trigger_character.as_deref())
        else {
            return Ok(None);
        };

        let Ok::<ZettelId, color_eyre::eyre::Error>(zid) = params
            .text_document_position
            .text_document
            .uri
            .to_file_path()
            .map_err(|()| eyre!("failed to turn into file path"))
            .and_then(TryInto::try_into)
        else {
            return Ok(None);
        };

        let responses = match trigger {
            // Link completion
            "[[" => ZettelEntity::load()
                .with(TagEntity)
                .all(&self.kt.db)
                .await
                .map_err(|_| jsonrpc::Error::internal_error())?
                .into_iter()
                .filter_map(|model| {
                    let z: Zettel = model.into();

                    // we dont want to return links to ourself
                    if z.id == zid {
                        return None;
                    }

                    let item = format!("{}|{}", z.id, z.title);
                    let detail = z
                        .front_matter(&self.kt.index)
                        .to_string()
                        .trim()
                        .to_string();

                    Some(CompletionItem::new_simple(item, detail))
                })
                .collect(),

            // Tag completion
            "@" => TagEntity::load()
                .all(&self.kt.db)
                .await
                .map_err(|_| jsonrpc::Error::internal_error())?
                .into_iter()
                .map(|model| {
                    let t: Tag = model.into();

                    let item = t.name.clone();

                    let detail = format!("{}\ncolor: {}", t.name, t.color);

                    CompletionItem::new_simple(item, detail)
                })
                .collect(),

            _ => return Ok(None),
        };

        return Ok(Some(CompletionResponse::Array(responses)));
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}
