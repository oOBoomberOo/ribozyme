# Ribozyme [![Crates.io](https://img.shields.io/crates/v/ribozyme)](https://crates.io/crates/ribozyme) [![Build Status](https://travis-ci.com/oOBoomberOo/ribozyme.svg?branch=master)](https://travis-ci.com/oOBoomberOo/ribozyme) [![Discord](https://img.shields.io/discord/428791010244558850?color=blue&label=Discord&logo=discord)](https://discord.gg/SnJQcfq)

## About

Ribozyme is a fast resourcepacks merger written in Rust.

## Installation

1) Install [Rustup](https://www.rust-lang.org/tools/install).
2) Install ribozyme via cargo: `cargo install ribozyme`.
3) Run `ribozyme --help`

## Usage

1. Open command line/terminal.
2. Run command `ribozyme path/to/input/directory path/to/output/directory`.
3. Enjoy your conflict-free resourcepack.

## Features

Ribozyme can:

1. Resolve model's override conflict.
2. Resolve language file conflict.
3. Auto-renaming duplicate file (and their references in other files as well).
