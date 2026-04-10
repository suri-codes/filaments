use std::fs;

use color_eyre::eyre::eyre;
use dto::{NanoId, TagEntity, ZettelEntity};
use tower_lsp::{
    Client, LanguageServer,
    jsonrpc::{self, Result},
    lsp_types::{
        CompletionItem, CompletionOptions, CompletionParams, CompletionResponse,
        GotoDefinitionParams, GotoDefinitionResponse, InitializeParams, InitializeResult,
        InitializedParams, Location, MessageType, OneOf, Range, ServerCapabilities,
        TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions, Url,
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
                definition_provider: Some(OneOf::Left(true)),

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

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let Ok(file_contents) = params
            .text_document_position_params
            .text_document
            .uri
            .to_file_path()
            .and_then(|path| fs::read_to_string(path).map_err(|_| ()))
        else {
            return Ok(None);
        };

        let line_idx = params.text_document_position_params.position.line;
        let char_idx = params.text_document_position_params.position.character;

        let Some(line) = file_contents.lines().nth(line_idx as usize) else {
            return Ok(None);
        };

        let id = extract_wikilink_id_at(line, char_idx as usize);

        match id {
            Some(id) => {
                let zod = self.kt.index.get_zod(&ZettelId::from(NanoId::from(id)));

                let uri = Url::parse(&format!("file:///{}", zod.path.display()))
                    .map_err(|e| tower_lsp::jsonrpc::Error::invalid_params(e.to_string()))?;

                Ok(Some(GotoDefinitionResponse::Scalar(Location {
                    uri,
                    range: Range::default(),
                })))
            }
            None => Ok(None),
        }
    }
}

/// Returns the `<id>` portion of a `[[<id> | <title>]]` wikilink if the
/// cursor (0-based char index) falls anywhere inside the `[[ ... ]]` span.
/// Also handles `[[<id>]]` (no pipe / title).
///
// helper function written by claude. I was lowkey too lazy to write this logic xd
fn extract_wikilink_id_at(line: &str, char_idx: usize) -> Option<&str> {
    let mut search_start = 0;

    while let Some(open) = line[search_start..].find("[[") {
        let open = open + search_start;
        let rest = &line[open + 2..];

        let Some(close) = rest.find("]]") else {
            break; // unclosed bracket — stop
        };
        let close_abs = open + 2 + close; // index of first `]`

        // Is the cursor inside [[ ... ]] ?
        if char_idx >= open && char_idx < close_abs + 2 {
            let inner = &line[open + 2..close_abs]; // text between [[ and ]]
            // id is everything before the first `|`, trimmed
            let id = inner.split('|').next().unwrap_or(inner).trim();
            return Some(id);
        }

        search_start = close_abs + 2; // advance past this `]]`
    }

    None
}
