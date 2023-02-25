use model::in_memory_index_model::{InMemoryIndexModel, Model};
use reader::plain_text_reader::{PlainTextReader, Reader};
use serde_json;
use std::{env, fs, path::Path, process::ExitCode};

mod lexer;
mod model;
mod reader;

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
            "help" | "h" => todo!("help information"),
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

            let dir = fs::read_dir(dir_path.as_str()).map_err(|err| {
                eprintln!("ERROR: could not open directory {dir_path} for indexing: {err}")
            })?;

            let mut model = InMemoryIndexModel::new();

            for path in dir {
                let file_path = path.map_err(|err| {
                    eprintln!("ERROR: could not read the file in directory: {dir_path} during indexing: {err}");
                })?.path();
                println!("File: {file_path:?}");
                let content = read_from_file(&file_path)?;

                model.add_document(file_path, &content.chars().collect::<Vec<char>>())?;
            }

            let output = serde_json::to_string(&model).map_err(|err| {
                eprintln!("ERROR: could not serialize the Index HashMap when indexing: {err}")
            })?;

            let index_path = "./src/index.json";
            fs::write(index_path, output).map_err(|err| {
                eprintln!(
                    "ERROR: could not write down serialized Index HashMap into the file: {index_path} when indexing: {err}"
                )
            })?;
        }
        "search" => {
            todo!();
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
        "txt" => {
            let r = PlainTextReader::read_text(file_path).map_err(|err| {
                eprintln!("ERROR: could not open the file when indexing: {err}");
            })?;

            return Ok(r);
        }
        _ => {
            eprintln!("The file type: {extension} is not be supported yet.");
            Err(())
        }
    }
}

fn prompt_usage(program: &str) {
    eprintln!("Usage: {program} [SUBCOMMAND] [OPTIONS]");
    eprintln!("Subcommands and options:");
    eprintln!("     index <folder>                    index the <folder> and save the index to index.json file.");
    eprintln!("     search <index-file> <query>       search <query> within the <index-file>");
}
