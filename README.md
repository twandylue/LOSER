# LOSER(LOcal Search Engine in Rust)

Local search engine in Rust, aka **LOSER**!

## Introduction

Using [`tf-idf`](https://en.wikipedia.org/wiki/Tf%E2%80%93idf) to find the related files.

## How to use

Help

```console
$ cargo run -- help
Usage: target/debug/serust [SUBCOMMAND] [OPTIONS]
Subcommands and options:
     index <folder>                    index the <folder> from scratch and save the index as `<folder>.loser.json` file.
     search <index-file> <query>       search <query> within the <index-file>
```

Create the index file for a folder
> You can find the `<folder>.loser.json` as a index file under the root directory of LOSER.

```console
$ cargo run -- index ./data
Indexing from scratch...
File path: ./data/test/test1.txt
File path: ./data/test2.txt
File path: ./data/pdf-sample.pdf
...
```

Search

```console
$ cargo run -- search ./data.loser.json github
File Path: ./data/test/test1.txt | Rank: 0.0029716683
File Path: ./data/test2.txt | Rank: 0
...
```

Web Server

```console
$ cargo run -- server ./data
INFO: listening at http://127.0.0.1:8080/
File Path: "./data/test2.txt"
...
Finished indexing...
```

Then go to [http://127.0.0.1:8080/](http://127.0.0.1:8080/), you can use the web browser to search the query.

## TODOs

- [x] UI(a simple web server)
- [x] Auto-indexing by folder(background service)
- [x] Extract text from PDF
- [ ] Extract text from XML
- [ ] Show indexing progress

## References

- [tf-idf](https://en.wikipedia.org/wiki/Tf%E2%80%93idf)
- [Search Engine in Rust](https://youtu.be/hm5xOJiVEeg)
