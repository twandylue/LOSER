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
> The `index.json` file would be under `./src/`

```console
$ cargo run -- index ./src/data
Indexing...
File: "./src/data/test1.txt"
File: "./src/data/test2.txt"
```

Search

```console
$ cargo run -- search ./src/index.json github
File Path: ./src/data/test1.txt | Rank: 0.0029716683
File Path: ./src/data/test2.txt | Rank: 0
```

## TODOs

- [ ] UI(Maybe a simple web server)
- [ ] Database for index files

## References

- [tf-idf](https://en.wikipedia.org/wiki/Tf%E2%80%93idf)
- [Search Engine in Rust](https://youtu.be/hm5xOJiVEeg)
