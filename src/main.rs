use model::in_memory_index_model::{InMemoryIndexModel, Model};
use reader::plain_text_reader::{PlainTextReader, Reader};
use serde::Deserialize;
use serde_json;
use std::{
    env, fs,
    io::BufWriter,
    path::Path,
    process::{exit, ExitCode},
};
use web_server::WebServer;

mod lexer;
mod model;
mod reader;
mod web_server;

fn main() -> ExitCode {
    match entry() {
        Ok(()) => ExitCode::SUCCESS,
        Err(()) => ExitCode::FAILURE,
    }
}

fn entry() -> Result<(), ()> {
    let mut args = env::args();
    let program = args.next().expect("path to program doesn't be provided.");
    let mut subcommand: Option<String> = None;
    if let Some(arg) = args.next() {
        match arg.as_str() {
            "help" | "h" => {
                prompt_usage(&program);
                exit(0);
            }
            _ => subcommand = Some(arg),
        }
    }

    let subcommand = subcommand.ok_or_else(|| {
        prompt_usage(&program);
        eprintln!("ERROR: no subcommand is provided.");
    })?;

    match subcommand.as_str() {
        "index" => {
            let dir_path = args.next().ok_or_else(|| {
                prompt_usage(&program);
                eprintln!("ERROR: no directory is provided for {subcommand} subcommand.");
            })?;

            println!("Indexing...");

            let mut model = InMemoryIndexModel::new();
            add_folder_to_model(dir_path, &mut model)?;

            let index_file = fs::File::create("index.json").map_err(|err| {
                eprintln!("ERROR: could not create the index file: {err}");
            })?;

            serde_json::to_writer(BufWriter::new(index_file), &model).map_err(|err| {
                eprintln!("ERROR: could not serialize index into the index file: {err}")
            })?;
        }
        "search" => {
            let index_path = args.next().ok_or_else(|| {
                prompt_usage(&program);
                eprintln!("ERROR: no path to index is provided {subcommand} subcommand.");
            })?;

            let query = args
                .next()
                .ok_or_else(|| {
                    prompt_usage(&program);
                    eprintln!("ERROR: no search query is provided {subcommand} subcommand.");
                })?
                .chars()
                .collect::<Vec<char>>();

            let index_file = fs::File::open(&index_path).map_err(|err| {
                eprintln!("ERROR: could not open the index file {index_path}: {err}");
            })?;

            let mut data = serde_json::Deserializer::from_reader(index_file);
            let model = InMemoryIndexModel::deserialize(&mut data).map_err(|err| {
                eprintln!("ERROR: could not parse the index file {index_path}: {err}");
            })?;

            for (path, rank) in model.search(&query)?.iter().take(10) {
                println!("File Path: {path} | Rank: {rank}", path = path.display());
            }

            return Ok(());
        }
        "server" => {
            let port = args.next().unwrap_or("8080".to_string());
            let addr = format!("127.0.0.1:{port}");

            let index_file = fs::File::open("index.json").map_err(|err| {
                eprintln!("ERROR: could not open the index file index.json: {err}");
            })?;

            let model: InMemoryIndexModel = serde_json::from_reader(index_file).map_err(|err| {
                eprintln!("ERROR: could not parse the index file index.json: {err}");
            })?;

            let server = WebServer::new(addr.as_str(), Box::new(model));

            server.start()?;
        }
        _ => {
            prompt_usage(&program);
            eprintln!("ERROR: unknown subcommand {subcommand}.");
        }
    }

    return Ok(());
}

fn read_from_file(file_path: &Path) -> Result<String, ()> {
    let extension = file_path
        .extension()
        .ok_or_else(|| {
            eprintln!("ERROR: could not detect file type of {file_path:?} without extension.")
        })?
        .to_string_lossy();

    match extension.as_ref() {
        "txt" => PlainTextReader::read_text(file_path),
        _ => {
            eprintln!("ERROR: The file type: {extension} has not been supported yet.");
            Err(())
        }
    }
}

fn add_folder_to_model(dir_path: String, model: &mut InMemoryIndexModel) -> Result<(), ()> {
    let dir = fs::read_dir(dir_path.as_str()).map_err(|err| {
        eprintln!("ERROR: could not open directory {dir_path} for indexing: {err}")
    })?;

    for path in dir {
        let file_path = path.map_err(|err| {
            eprintln!("ERROR: could not read the file in directory: {dir_path} during indexing: {err}"); 
        })?.path();

        let last_modified = file_path
            .metadata()
            .map_err(|err| {
                eprintln!(
                    "ERROR: could not get the metadata of file {file_path}: {err}",
                    file_path = file_path.display()
                )
            })?
            .modified()
            .map_err(|err| {
                eprintln!(
                    "ERROR: could not get the last modified time of file {file_path}: {err}",
                    file_path = file_path.display()
                )
            })?;

        if file_path.is_dir() {
            add_folder_to_model(file_path.to_string_lossy().to_string(), model)?
        } else if model.requires_reindexing(&file_path, last_modified) {
            match read_from_file(&file_path) {
                Ok(content) => {
                    println!("File: {file_path:?}");

                    model.add_document(
                        file_path,
                        &content.chars().collect::<Vec<char>>(),
                        last_modified,
                    )?;
                }
                Err(()) => continue,
            }
        }
    }

    Ok(())
}

fn prompt_usage(program: &str) {
    eprintln!("Usage: {program} [SUBCOMMAND] [OPTIONS]");
    eprintln!("Subcommands and options:");
    eprintln!("     index <folder>                    index the <folder> and save the index to index.json file.");
    eprintln!("     search <index-file> <query>       search <query> within the <index-file>");
}
