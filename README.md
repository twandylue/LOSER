# LOSER(LOcal Search Engine in Rust)

Local search engine in Rust, aka **LOSER**!

## Introduction

Using `tf-idf` to find the related files.

## How to use

Help

```console
$ cargo run -- help
Usage: target/debug/serust [SUBCOMMAND] [OPTIONS]
Subcommands and options:
     index <folder>                    index the <folder> and save the index to index.json file.
     search <index-file> <query>       search <query> within the <index-file>
```

Create the index for a folder
> You can find the `index.json` under the root directory of LOSER.

```console
$ cargo run -- index ./data
Indexing...
File: "./data/test/test1.txt"
File: "./data/test2.txt"
```

Search

```console
$ cargo run -- search ./data.loser.json github
File Path: ./data/test/test1.txt | Rank: 0.0029716683
File Path: ./data/test2.txt | Rank: 0
```

Web Server

```console
$ cargo run -- server ./data
INFO: listening at http://127.0.0.1:8080/
File: "./data/test2.txt"
Finished indexing...
```

Then go to [http://127.0.0.1:8080/](http://127.0.0.1:8080/), you can use the web browser to search the query.

## TODOs

- [x] UI(a simple web server)
- [x] Auto-indexing by folder(background service)
- [x] Extract text from PDF
- [ ] Extract text from XML

## References

- [tf-idf](https://en.wikipedia.org/wiki/Tf%E2%80%93idf)
- [Search Engine in Rust](https://youtu.be/hm5xOJiVEeg)
