# Ribozyme

## About

Ribozyme is a blazingly fast resourcepacks merger written in Rust.

## Installation

1) Install [Rustup](https://www.rust-lang.org/tools/install).
2) Install ribozyme via cargo: `cargo install ribozyme`.
3) Run `ribozyme --help`

## Usage

1) Open terminal/command line.
2) Run `ribozyme <directory>` command where `<directory>` is a path to a folder containing resourcepacks you want to merge.
3) You will be asked to choose which resourcepacks to merge, by default it will merge every packs inside that folder.
4) You will be asked to choose the name of the output file.
5) Your merged resourcepacks is ready.!

## Features

- Merging `custom_model_id`. Note that if there is a conflicting `custom_model_id` this tool cannot help with that.
- Merging language files.
