use std::io::Read;

const CHUNK_SIZE: usize = 16;

const LOG_FILE: &str = "D:/projs/loki/lsp/log.txt";

fn main() {
	match run() {
		Ok(_) => (),
		Err(e) => {
			std::fs::write("D:/projs/loki/lsp/err.txt", e).unwrap();
		},
	}
}

// TODO: handle case where just reading a single chunk, starts reading the next thing
//       this can be solved by doing something similiar to https://github.com/mozilla/web-ext/blob/master/src/firefox/rdp-client.js
fn run() -> Result<(), String> {
	const BUF_MIN_LEN: usize = "Content-Length: ____\r\n\r\n".len();

	loop {
		let mut buf = vec![0u8; BUF_MIN_LEN];
		let mut n = 0;
	
		let start_of_content = loop {
			if let Some(pos) = finished_headers(&buf) {
				break pos;
			}
	
			n += std::io::stdin().read(&mut buf[n..]).map_err(|e| format!("{e} at {}:{}:{}", file!(), line!(), column!()))?;
			if buf.len() == n {
				buf.resize(n + CHUNK_SIZE, 0);
			}
		};
	
		let headers = parse_headers(&buf[..start_of_content])?;
		let content_len = {
			let h = headers.iter().find(|(k, _)| k == "Content-Length").ok_or("Couldn't find Content-Length")?;
			h.1.parse::<usize>().map_err(|e| e.to_string() + " value is |" + &h.1 + "|")?
		};
	
		buf.resize(start_of_content + content_len, 0);
		std::io::stdin().read_exact(&mut buf[n..]).map_err(|e| format!("{e} at {}:{}:{}", file!(), line!(), column!()))?;
		
		let msg = serde_json::from_slice::<serde_json::Value>(&buf[start_of_content..]).map_err(|e| format!("{e} at {}:{}:{}", file!(), line!(), column!()))?;
		write!(
			std::fs::File::options().create(true).append(true).open(LOG_FILE).map_err(|e| format!("{e} at {}:{}:{}", file!(), line!(), column!()))?,
			"Recieved {msg:#}\n"
		).map_err(|e| format!("{e} at {}:{}:{}", file!(), line!(), column!()))?;

		if msg["id"].is_null() {
			process_notification(serde_json::from_value(msg).map_err(|e| format!("{e} at {}:{}:{}", file!(), line!(), column!()))?)?;
		} else {
			process_request(serde_json::from_value(msg).map_err(|e| format!("{e} at {}:{}:{}", file!(), line!(), column!()))?)?;
		}
		

		// let client_info = data.params.client_info.ok_or("No client info")?;
		// std::fs::write("D:/projs/loki/lsp/log.txt", client_info.name).map_err(|e| e.to_string())?;
	}

	// Ok(())
}

fn send_jrpc(json: String) -> std::io::Result<()> {
	write!(
		std::fs::File::options().create(true).append(true).open(LOG_FILE)?,
		"Sending {json:#}\n"
	)?;

	write!(std::io::stdout(), "Content-Length: {len}\r\n\r\n{json}", len = json.len())?;
	Ok(())
}

fn finished_headers(buf: &[u8]) -> Option<usize> {
	buf.windows(4).enumerate().find(|(_, w)| w == b"\r\n\r\n").map(|(i, _)| i + 4)
}

fn parse_headers(buf: &[u8]) -> Result<Vec<(String, String)>, String> {
	let s = std::str::from_utf8(buf).map_err(|e| format!("{e} at {}:{}:{}", file!(), line!(), column!()))?;
	Ok(s.trim().split("\r\n")
		.map(|h| h.split_once(": "))
		.collect::<Option<Vec<_>>>().ok_or("Missing : in one of the headers\n".to_string() + s)?.into_iter()
		.map(|(k, v)| (k.to_string(), v.to_string()))
		.collect())
}

use lsp::*;
use std::io::Write;
fn process_notification(notification: LspNotification) -> Result<(), String> {
	match notification.method.as_str() {
		"exit" => std::process::exit(0),

		m => {
			send_jrpc(serde_json::json!({
				"jsonrpc": "2.0",
				"method": "window/logMessage",
				"params": {
					"type": 2,
					"message": format!("Unknown notification {m}"),
				},
			}).to_string()).map_err(|e| format!("{e} at {}:{}:{}", file!(), line!(), column!()))?;
		},
	}
	Ok(())
}

fn process_request(request: LspRequest) -> Result<(), String> {
	match request.method.as_str() {
		"initialize" => {
			send_jrpc(serde_json::to_string(
				&InitializeResponse {
					jsonrpc: "2.0".to_string(),
					id: request.id,
					error: None,
					result: Some(InitializeResult {
						server_info: Some(InitializeResult__ServerInfo {
							name: "Loki Language Server".to_string(),
							version: None,
						}),

						capabilities: InitializeResult__ServerCapabilities {
							..Default::default()
						},
					}),
				}
			).map_err(|e| format!("{e} at {}:{}:{}", file!(), line!(), column!()))?).map_err(|e| format!("{e} at {}:{}:{}", file!(), line!(), column!()))?;

			// send_jrpc(serde_json::json!({
			// 	"jsonrpc": "2.0",
			// 	"method": "window/logMessage",
			// 	"params": {
			// 		"type": 2,
			// 		"message": "PLEASE LOG THIS",
			// 	},
			// }).to_string()).map_err(|e| format!("{e} at {}:{}:{}", file!(), line!(), column!()))?;
		},

		"shutdown" => send_jrpc(serde_json::to_string(&LspResponse {
			jsonrpc: "2.0".to_string(),
			id: request.id,
			result: None,
			error: None,
		}).map_err(|e| format!("{e} at {}:{}:{}", file!(), line!(), column!()))?).map_err(|e| format!("{e} at {}:{}:{}", file!(), line!(), column!()))?,

		m => {
			// write!(
			// 	std::fs::File::options().create(true).append(true).open(LOG_FILE).map_err(|e| e.to_string())?,
			// 	"Unknown request method: {m}"
			// ).map_err(|e| e.to_string())?;

			send_jrpc(serde_json::to_string(&LspError {
				jsonrpc: "2.0".to_string(),
				id: request.id,
				result: None,
				error: Some(jrpc::ResponseError {
					// Method not found
					code: -32601,
					message: format!("Method {m} not found"),
					data: None,
				}),
			}).map_err(|e| format!("{e} at {}:{}:{}", file!(), line!(), column!()))?).map_err(|e| format!("{e} at {}:{}:{}", file!(), line!(), column!()))?;
		},
	}

	Ok(())
}

mod jrpc {
	use serde_json::Value;
	use serde::{Deserialize, Serialize};

	// #[macro_export]
	// macro_rules! jrpc_msg {
	// 	($name:ident  $( $( #[doc = $doc:literal] )* $( #[serde $serde_args:tt] )? $f:ident : $t:ty ),* $(,)? ) => {
	// 		#[derive(serde::Deserialize)]
	// 		#[allow(dead_code)]
	// 		pub struct $name {
	// 			pub jsonrpc: String,
	// 			$(
	// 				$( #[doc = $doc] )*
	// 				$( #[serde $serde_args] )?
	// 				pub $f : $t
	// 			),*
	// 		}
	// 	}
	// }
	#[macro_export]
	macro_rules! jrpc_msg {
		($name:ident  $( $f:ident : $t:ty ),* $(,)? ) => {
			#[derive(serde::Deserialize, serde::Serialize)]
			#[allow(dead_code)]
			pub struct $name {
				pub jsonrpc: String,
				$( pub $f : $t ),*
			}
		}
	}
	
	#[macro_export]
	macro_rules! jrpc_request {
		($name:ident, $params:ty) => ($crate::jrpc_msg! {
			$name
			// i32 | String
			id: serde_json::Value,
			method: String,
			// array | object
			// although params is marked as ?, requests explicitly say that params is none
			params: $params,
		})
	}

	#[derive(Deserialize, Serialize)]
	pub struct ResponseError {
		pub code: i32,
		pub message: String,
		// String | i32 | bool | [] | object | null
		pub data: Option<Value>,
	}

	// #[macro_export]
	// macro_rules! jrpc_response {
	// 	($name:ident $( $( #[doc = $doc:literal] )* $( #[serde $serde_args:tt] )? $f:ident : $t:ty ),* $(,)? ) => ($crate::jrpc_msg! {
	// 		$name
	// 		id: serde_json::Value,
	// 		// Option<String | i32 | bool | [] | object | null>
	// 		result: Option<serde_json::Value>,
	// 		error: Option<$crate::jrpc::ResponseError>,
	// 		$( $( #[doc = $doc] )* $( #[serde $serde_args] )?  $f : $t ),*
	// 	})
	// }
	#[macro_export]
	macro_rules! jrpc_response {
		($name:ident, $result:ty) => ($crate::jrpc_msg! {
			$name
			id: serde_json::Value,
			// Option<String | i32 | bool | [] | object | null>
			// result: Option<serde_json::Value>,
			result: Option<$result>,
			error: Option<$crate::jrpc::ResponseError>,
		})
	}

	#[macro_export]
	macro_rules! jrpc_notification {
		($name:ident, $params:ty ) => ($crate::jrpc_msg! {
			$name
			method: String,
			// params: Option<array | object>,
			params: $params,
		})
	}
}

mod lsp {
	#![allow(non_camel_case_types)]
	
	use crate::{jrpc_request, jrpc_response, jrpc_notification};
	use serde::{Deserialize, Serialize};
	use serde_json::Value;

	jrpc_request! { LspRequest, Option<Value> }
	jrpc_notification! { LspNotification, Option<Value> }
	jrpc_response! { LspError, () }
	jrpc_response! { LspResponse, () }

	// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#initialize
	jrpc_request! { InitializeRequest, InitializeParams }
	#[derive(Deserialize, Serialize)]
	pub struct InitializeParams {
		// integer | null
		// don't care if processId was just missing or is null
		/// The process Id of the parent process that started the server. Is null if
		/// the process has not been started by another process. If the parent
		/// process is not alive then the server should exit (see exit notification)
		/// its process.
		#[serde(rename = "processId")]
		pub process_id: Option<i32>,

		/// Information about the client
		/// @since 3.15.0
		#[serde(rename = "clientInfo")]
		pub client_info: Option<InitializeParams__ClientInfo>,
		
		/// The locale the client is currently showing the user interface
		/// in. This must not necessarily be the locale of the operating
		/// system.
		///
		/// Uses IETF language tags as the value's syntax
		/// (See https://en.wikipedia.org/wiki/IETF_language_tag)
		///
		/// @since 3.16.0
		pub locale: Option<String>,

		/// The rootPath of the workspace. Is null
		/// if no folder is open.
		///
		/// @deprecated in favour of `rootUri`.
		#[serde(rename = "rootPath")]
		pub root_path: Option<String>,

		/// The rootUri of the workspace. Is null if no
		/// folder is open. If both `rootPath` and `rootUri` are set
		/// `rootUri` wins.
		///
		/// @deprecated in favour of `workspaceFolders`
		// pub rootUri: Option<DocumentUri>,

		/// User provided initialization options.
		// pub initializationOptions: Option<LSPAny>,

		/// The capabilities provided by the client (editor or tool)
		// pub capabilities: ClientCapabilities,

		/// The initial trace setting. If omitted trace is disabled ('off').
		// pub trace: Option<TraceValue>,

		/// The workspace folders configured in the client when the server starts.
		/// This property is only available if the client supports workspace folders.
		/// It can be `null` if the client supports workspace folders but none are
		/// configured.
		///
		/// @since 3.6.0
		// pub workspaceFolders: Option<WorkspaceFolder[]>,

		__: Option<()>,
	}
	
	#[derive(Deserialize, Serialize)]
	pub struct InitializeParams__ClientInfo {
		/// The name of the client as defined by the client.
		pub name: String,
		/// The client's version as defined by the client.
		pub version: Option<String>,
	}

	jrpc_response! { InitializeResponse, InitializeResult }
	#[derive(Deserialize, Serialize)]
	pub struct InitializeResult {
		/// The capabilities the language server provides.
		pub capabilities: InitializeResult__ServerCapabilities,
	
		/// Information about the server.
		/// 
		/// @since 3.15.0
		#[serde(rename = "serverInfo")]
		pub server_info: Option<InitializeResult__ServerInfo>,
	}

	#[derive(Deserialize, Serialize, Default)]
	#[allow(non_snake_case)] // FIXME:
	pub struct InitializeResult__ServerCapabilities {
		/// The position encoding the server picked from the encodings offered
		/// by the client via the client capability `general.positionEncodings`.
		///
		/// If the client didn't provide any position encodings the only valid
		/// value that a server can return is 'utf-16'.
		///
		/// If omitted it defaults to 'utf-16'.
		///
		/// @since 3.17.0
		// pub positionEncoding: Option<PositionEncodingKind>,
		pub positionEncoding: Option<()>,
	
		/// Defines how text documents are synced. Is either a detailed structure
		/// defining each notification or for backwards compatibility the
		/// TextDocumentSyncKind number. If omitted it defaults to
		/// `TextDocumentSyncKind.None`.
		// pub textDocumentSync: Option<TextDocumentSyncOptions | TextDocumentSyncKind>,
		pub textDocumentSync: Option<()>,
	
		/// Defines how notebook documents are synced.
		///
		/// @since 3.17.0
		// pub notebookDocumentSync: Option<NotebookDocumentSyncOptions | NotebookDocumentSyncRegistrationOptions>,
		pub notebookDocumentSync: Option<()>,
	
		/// The server provides completion support.
		// pub completionProvider: Option<CompletionOptions>,
		pub completionProvider: Option<()>,
	
		/// The server provides hover support.
		// pub hoverProvider: Option<boolean | HoverOptions>,
		pub hoverProvider: Option<()>,
	
		/// The server provides signature help support.
		// pub signatureHelpProvider: Option<SignatureHelpOptions>,
		pub signatureHelpProvider: Option<()>,
	
		/// The server provides go to declaration support.
		///
		/// @since 3.14.0
		// pub declarationProvider: Option<boolean | DeclarationOptions | DeclarationRegistrationOptions>,
		pub declarationProvider: Option<()>,
	
		/// The server provides goto definition support.
		// pub definitionProvider: Option<boolean | DefinitionOptions>,
		pub definitionProvider: Option<()>,
	
		/// The server provides goto type definition support.
		///
		/// @since 3.6.0
		// pub typeDefinitionProvider: Option<boolean | TypeDefinitionOptions | TypeDefinitionRegistrationOptions>,
		pub typeDefinitionProvider: Option<()>,
	
		/// The server provides goto implementation support.
		///
		/// @since 3.6.0
		// pub implementationProvider: Option<boolean | ImplementationOptions | ImplementationRegistrationOptions>,
		pub implementationProvider: Option<()>,
	
		/// The server provides find references support.
		// pub referencesProvider: Option<boolean | ReferenceOptions>,
		pub referencesProvider: Option<()>,
	
		/// The server provides document highlight support.
		// pub documentHighlightProvider: Option<boolean | DocumentHighlightOptions>,
		pub documentHighlightProvider: Option<()>,
	
		/// The server provides document symbol support.
		// pub documentSymbolProvider: Option<boolean | DocumentSymbolOptions>,
		pub documentSymbolProvider: Option<()>,
	
		/// The server provides code actions. The `CodeActionOptions` return type is
		/// only valid if the client signals code action literal support via the
		/// property `textDocument.codeAction.codeActionLiteralSupport`.
		// pub codeActionProvider: Option<boolean | CodeActionOptions>,
		pub codeActionProvider: Option<()>,
	
		/// The server provides code lens.
		// pub codeLensProvider: Option<CodeLensOptions>,
		pub codeLensProvider: Option<()>,
	
		/// The server provides document link support.
		// pub documentLinkProvider: Option<DocumentLinkOptions>,
		pub documentLinkProvider: Option<()>,
	
		/// The server provides color provider support.
		///
		/// @since 3.6.0
		// pub colorProvider: Option<boolean | DocumentColorOptions | DocumentColorRegistrationOptions>,
		pub colorProvider: Option<()>,
	
		/// The server provides document formatting.
		// pub documentFormattingProvider: Option<boolean | DocumentFormattingOptions>,
		pub documentFormattingProvider: Option<()>,
	
		/// The server provides document range formatting.
		// pub documentRangeFormattingProvider: Option<boolean | DocumentRangeFormattingOptions>,
		pub documentRangeFormattingProvider: Option<()>,
	
		/// The server provides document formatting on typing.
		// pub documentOnTypeFormattingProvider: Option<DocumentOnTypeFormattingOptions>,
		pub documentOnTypeFormattingProvider: Option<()>,
	
		/// The server provides rename support. RenameOptions may only be
		/// specified if the client states that it supports
		/// `prepareSupport` in its initial `initialize` request.
		// pub renameProvider: Option<boolean | RenameOptions>,
		pub renameProvider: Option<()>,
	
		/// The server provides folding provider support.
		///
		/// @since 3.10.0
		// pub foldingRangeProvider: Option<boolean | FoldingRangeOptions | FoldingRangeRegistrationOptions>,
		pub foldingRangeProvider: Option<()>,
	
		/// The server provides execute command support.
		// pub executeCommandProvider: Option<ExecuteCommandOptions>,
		pub executeCommandProvider: Option<()>,
	
		/// The server provides selection range support.
		///
		/// @since 3.15.0
		// pub selectionRangeProvider: Option<boolean | SelectionRangeOptions | SelectionRangeRegistrationOptions>,
		pub selectionRangeProvider: Option<()>,
	
		/// The server provides linked editing range support.
		///
		/// @since 3.16.0
		// pub linkedEditingRangeProvider: Option<boolean | LinkedEditingRangeOptions | LinkedEditingRangeRegistrationOptions>,
		pub linkedEditingRangeProvider: Option<()>,
	
		/// The server provides call hierarchy support.
		///
		/// @since 3.16.0
		// pub callHierarchyProvider: Option<boolean | CallHierarchyOptions | CallHierarchyRegistrationOptions>,
		pub callHierarchyProvider: Option<()>,
	
		/// The server provides semantic tokens support.
		///
		/// @since 3.16.0
		// pub semanticTokensProvider: Option<SemanticTokensOptions | SemanticTokensRegistrationOptions>,
		pub semanticTokensProvider: Option<()>,
	
		/// Whether server provides moniker support.
		///
		/// @since 3.16.0
		// pub monikerProvider: Option<boolean | MonikerOptions | MonikerRegistrationOptions>,
		pub monikerProvider: Option<()>,
	
		/// The server provides type hierarchy support.
		///
		/// @since 3.17.0
		// pub typeHierarchyProvider: Option<boolean | TypeHierarchyOptions | TypeHierarchyRegistrationOptions>,
		pub typeHierarchyProvider: Option<()>,
	
		/// The server provides inline values.
		///
		/// @since 3.17.0
		// pub inlineValueProvider: Option<boolean | InlineValueOptions | InlineValueRegistrationOptions>,
		pub inlineValueProvider: Option<()>,
	
		/// The server provides inlay hints.
		///
		/// @since 3.17.0
		// pub inlayHintProvider: Option<boolean | InlayHintOptions | InlayHintRegistrationOptions>,
		pub inlayHintProvider: Option<()>,
	
		/// The server has support for pull model diagnostics.
		///
		/// @since 3.17.0
		// pub diagnosticProvider: Option<DiagnosticOptions | DiagnosticRegistrationOptions>,
		pub diagnosticProvider: Option<()>,
	
		/// The server provides workspace symbol support.
		// pub workspaceSymbolProvider: Option<boolean | WorkspaceSymbolOptions>,
		pub workspaceSymbolProvider: Option<()>,
	
		/// Workspace specific server capabilities
		pub workspace: Option<InitializeResult__ServerCapabilities__Workspace>,
	
		/// Experimental server capabilities.
		pub experimental: Option<Value>,
	}

	#[derive(Deserialize, Serialize)]
	#[allow(non_snake_case)] // FIXME:
	pub struct InitializeResult__ServerCapabilities__Workspace {
		/// The server supports workspace folder.
		///
		/// @since 3.6.0
		// pub workspaceFolders: Option<WorkspaceFoldersServerCapabilities>,
		pub workspaceFolders: Option<()>,

		/// The server is interested in file notifications/requests.
		///
		/// @since 3.16.0
		pub fileOperations: Option<InitializeResult__ServerCapabilities__Workspace__FileOperations>,
	}

	#[derive(Deserialize, Serialize)]
	#[allow(non_snake_case)] // FIXME:
	pub struct InitializeResult__ServerCapabilities__Workspace__FileOperations {
		/// The server is interested in receiving didCreateFiles
		/// notifications.
		// pub didCreate: Option<FileOperationRegistrationOptions>,
		pub didCreate: Option<()>,

		/// The server is interested in receiving willCreateFiles requests.
		// pub willCreate: Option<FileOperationRegistrationOptions>,
		pub willCreate: Option<()>,

		/// The server is interested in receiving didRenameFiles
		/// notifications.
		// pub didRename: Option<FileOperationRegistrationOptions>,
		pub didRename: Option<()>,

		/// The server is interested in receiving willRenameFiles requests.
		// pub willRename: Option<FileOperationRegistrationOptions>,
		pub willRename: Option<()>,

		/// The server is interested in receiving didDeleteFiles file
		/// notifications.
		// pub didDelete: Option<FileOperationRegistrationOptions>,
		pub didDelete: Option<()>,

		/// The server is interested in receiving willDeleteFiles file
		/// requests.
		// pub willDelete: Option<FileOperationRegistrationOptions>,
		pub willDelete: Option<()>,
	}

	#[derive(Deserialize, Serialize)]
	pub struct InitializeResult__ServerInfo {
		/// The name of the server as defined by the server.
		pub name: String,
		/// The server's version as defined by the server.
		pub version: Option<String>,
	}
}
