use smc_assembler::assembler::backends::Backend as SmcBackend;
use smc_assembler::assembler::{Assembler, AssemblerError, LabelMap};
use smc_assembler::lexer::{Lexer, LexerError};
use smc_assembler::parser::{DefineMap, Parser, ParserError};
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct Definitions {
    defines: DefineMap,
    labels: LabelMap,
}

#[derive(Debug)]
struct Backend {
    client: Client,
    definitions: RwLock<Option<Definitions>>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: None,
                    all_commit_characters: None,
                    work_done_progress_options: Default::default(),
                    completion_item: Some(CompletionOptionsCompletionItem {
                        label_details_support: Some(true),
                    }),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        identifier: None,
                        inter_file_dependencies: false,
                        workspace_diagnostics: false,
                        work_done_progress_options: Default::default(),
                    },
                )),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let position = params.text_document_position_params.position;
        self.client
            .log_message(MessageType::INFO, format!("position is {:?}", position))
            .await;

        // let location = Location::new(
        //     params.text_document_position_params.text_document.uri,
        //     Range::new(start, end),
        // );

        // Ok(Some(GotoDefinitionResponse::Scalar(location)))
        Ok(None)
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn completion(&self, _: CompletionParams) -> Result<Option<CompletionResponse>> {
        self.client
            .log_message(MessageType::INFO, "completion request")
            .await;
        Ok(Some(CompletionResponse::Array(vec![])))
    }

    async fn hover(&self, _: HoverParams) -> Result<Option<Hover>> {
        self.client
            .log_message(MessageType::INFO, "hover occurred")
            .await;
        Ok(None)
    }

    async fn shutdown(&self) -> Result<()> {
        self.client
            .log_message(MessageType::INFO, "Shutdown initiated")
            .await;
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.on_change(TextDocumentChange {
            uri: params.text_document.uri.to_string(),
            text: &params.text_document.text,
        })
        .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.on_change(TextDocumentChange {
            text: &params.content_changes[0].text,
            uri: params.text_document.uri.to_string(),
        })
        .await;
    }
}

impl Backend {
    async fn on_change(&self, item: TextDocumentChange<'_>) {
        // self.client.send
        // Url::parse(&item.uri)
        let backend = (|| {
            match Url::parse(&item.uri) {
                Ok(url) => match url.to_file_path() {
                    Ok(path) => {
                        if let Some(ext) = path.extension() {
                            if ext.eq_ignore_ascii_case("tasm") {
                                return Some(SmcBackend::TauAnalyzersNone);
                            } else if ext.eq_ignore_ascii_case("smc") {
                                return Some(SmcBackend::BatPU2);
                            }

                            return None;
                        }
                    }
                    Err(_) => return None,
                },
                Err(_) => return None,
            }
            return None;
        })();

        let backend = match backend {
            Some(backend) => backend,
            None => return,
        };

        let tokens: Vec<_> = Lexer::new(&item.text).into_iter().collect();
        let parsed = Parser::new(tokens).parse();
        let assembler = Assembler::new(backend, parsed).assemble();

        {
            let mut definitions = self.definitions.write().await;
            *definitions = Some(Definitions {
                defines: assembler.defines,
                labels: assembler.labels,
            });
        }

        let errors = match assembler.result {
            Ok(_) => return,
            Err(errs) => errs,
        };

        let diagnostics: Vec<_> = errors
            .into_iter()
            .map(|err| {
                let span = match &err {
                    AssemblerError::DefineNotFound(span, _) => span,
                    AssemblerError::LabelNotFound(span, _) => span,
                    AssemblerError::ParserError(parser_error) => match parser_error {
                        ParserError::SyntaxError(lexer_error) => match lexer_error {
                            LexerError::InvalidNumber(span, _) => span,
                            LexerError::UnexpectedCharacter(span, _) => span,
                            LexerError::ExpectedCharacter(span, _) => span,
                            LexerError::UnknownCondition(span, _) => span,
                            LexerError::InvalidOffset(span, _) => span,
                            LexerError::InvalidIsaCode(span, _) => span,
                            LexerError::InvalidRegisterNumber(span, _) => span,
                        },
                        ParserError::DuplicateDefine(span, _) => span,
                        ParserError::DuplicateLabel(span, _) => span,
                        ParserError::ExpectedButReceived(span, _, _) => span,
                        ParserError::UnexpectedEof(span) => span,
                        ParserError::InvalidSkip(span, _) => span,
                    },
                    AssemblerError::UnsupportedOperation(span, _) => span,
                    AssemblerError::InvalidRegister(span, _) => span,
                    AssemblerError::AddressOutOfRange(span, _) => span,
                    AssemblerError::OffsetOutOfRange(span, _) => span,
                    AssemblerError::InvalidCondition(span, _) => span,
                    AssemblerError::ImmediateOutOfRange(span, _) => span,
                };

                let start = span.start_location(item.text);
                let start_position = Position::new(
                    (start.0 as u32).saturating_sub(1),
                    (start.1 as u32).saturating_sub(1),
                );

                let end = span.end_location(item.text);
                let end_position = Position::new(
                    (end.0 as u32).saturating_sub(1),
                    (end.1 as u32).saturating_sub(1),
                );

                Diagnostic {
                    range: Range::new(start_position, end_position),
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: None,
                    code_description: None,
                    source: None,
                    message: err.to_string(),
                    related_information: None,
                    tags: None,
                    data: None,
                }
            })
            .collect();

        let uri =
            Url::parse(&item.uri).unwrap_or_else(|_| Url::from_directory_path(&item.uri).unwrap());
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }
}

struct TextDocumentChange<'a> {
    uri: String,
    text: &'a str,
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        definitions: RwLock::new(None),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
