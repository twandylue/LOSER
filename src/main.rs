use model::in_memory_index_model::{InMemoryIndexModel, Model};
use reader::{pdf_reader::PDFReader, plain_text_reader::PlainTextReader, reader_trait::Reader};
use serde::Deserialize;
use serde_json;
use std::{
    env, fs,
    io::BufWriter,
    path::{Path, PathBuf},
    process::{exit, ExitCode},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
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

            let folder_name = Path::new(&dir_path)
                .file_name()
                .ok_or_else(|| eprintln!("ERROR: could not extract the folder name: {dir_path}"))?;

            let output_file_name = format!(
                "{folder_name}.loser.json",
                folder_name = folder_name.to_string_lossy().to_string()
            );

            println!("Indexing from scratch...");

            let model = Arc::new(Mutex::new(InMemoryIndexModel::new()));
            add_folder_to_model(&dir_path, Arc::clone(&model))?;

            let model = &*model.lock().unwrap();
            save_mode_as_json(model, &Path::new(&output_file_name))?;
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
                    eprintln!("ERROR: no search query is provided for {subcommand} subcommand.");
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
            let dir_path = args.next().ok_or_else(|| {
                prompt_usage(&program);
                eprintln!("ERROR: no folder is provided for {subcommand} subcommand.")
            })?;
            let index_path = Path::new(&format!("{dir_path}.loser.json")).to_path_buf();

            let port = args.next().unwrap_or("8080".to_string());
            let addr = format!("127.0.0.1:{port}");

            let is_existed = index_path.try_exists().map_err(|err| {
                eprintln!(
                    "ERROR: could not check the existence of file {index_path}: {err}",
                    index_path = index_path.display()
                )
            })?;

            // TODO: how to make it more generic?
            let model: Arc<Mutex<InMemoryIndexModel>>;

            if is_existed {
                let index_file = fs::File::open(&index_path).map_err(|err| {
                    eprintln!(
                        "ERROR: could not open the index file {index_path}: {err}",
                        index_path = index_path.display()
                    )
                })?;

                model = Arc::new(Mutex::<InMemoryIndexModel>::new(
                    serde_json::from_reader(&index_file).map_err(|err| {
                        eprintln!(
                            "ERROR: could not parse the index file {index_path}: {err}",
                            index_path = index_path.display()
                        )
                    })?,
                ));
            } else {
                model = Arc::new(Mutex::<InMemoryIndexModel>::new(Default::default()));
            }

            {
                // TODO: what to do if this thread broken?
                let model = Arc::clone(&model);

                thread::spawn(move || -> Result<(), ()> {
                    loop {
                        // TODO: checking if the files existed need to be refactored
                        let mut removed_files: Vec<PathBuf> = Vec::new();
                        for path in model.lock().unwrap().docs.keys() {
                            if path.try_exists().map_err(|err| {
                                eprintln!(
                                    "ERROR: could not check if the file {path} is existed: {err}",
                                    path = path.display()
                                )
                            })? {
                                continue;
                            }

                            println!("{file} does not existed anymore", file = path.display());
                            removed_files.push(path.clone());
                        }

                        for file in removed_files {
                            model.lock().unwrap().remove_document(&file)
                        }

                        add_folder_to_model(
                            &Path::new(&dir_path).to_string_lossy().to_string(),
                            Arc::clone(&model),
                        )?;
                        let model = model.lock().unwrap();
                        save_mode_as_json(&model, &index_path)?;

                        drop(model);
                        println!("Finished indexing...");

                        thread::sleep(Duration::from_secs(1));
                    }
                });
            }

            let server = WebServer::new(addr.as_str(), model);

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
        "txt" => PlainTextReader::read_text(&file_path),
        "pdf" => PDFReader::read_text(&file_path),
        "xml" => todo!(),
        _ => {
            eprintln!("ERROR: The file type: {extension} has not been supported yet.");
            Err(())
        }
    }
}

fn add_folder_to_model(dir_path: &str, model: Arc<Mutex<InMemoryIndexModel>>) -> Result<(), ()> {
    let dir = fs::read_dir(dir_path).map_err(|err| {
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
            add_folder_to_model(&file_path.to_string_lossy().to_string(), Arc::clone(&model))?
        } else if model
            .lock()
            .unwrap()
            .requires_reindexing(&file_path, last_modified)
        {
            match read_from_file(&file_path) {
                Ok(content) => {
                    println!("File path: {file_path}", file_path = file_path.display());

                    model.lock().unwrap().add_document(
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

fn save_mode_as_json(model: &InMemoryIndexModel, file_path: &Path) -> Result<(), ()> {
    let file = fs::File::create(file_path).map_err(|err| {
        eprintln!(
            "ERROR: could not create the index file: {file_path}: {err}",
            file_path = file_path.display()
        )
    })?;

    serde_json::to_writer(BufWriter::new(file), model)
        .map_err(|err| eprintln!("ERROR: could not serialize index into the index file: {err}"))?;

    Ok(())
}

fn prompt_usage(program: &str) {
    eprintln!("Usage: {program} [SUBCOMMAND] [OPTIONS]");
    eprintln!("Subcommands and options:");
    eprintln!("     index <folder>                    index the <folder> and save the index to index.json file.");
    eprintln!("     search <index-file> <query>       search <query> within the <index-file>");
    eprintln!("     server <folder> [port]            search on local HTTP server within files in <folder>");
}
