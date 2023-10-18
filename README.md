![CI](https://github.com/rive-app/rive-rs/actions/workflows/ci.yml/badge.svg)
[![crates.io](https://img.shields.io/crates/v/rive-rs.svg)](https://crates.io/crates/rive-rs)
[![docs.rs](https://img.shields.io/docsrs/rive-rs)](https://docs.rs/rive-rs)
![Discord badge](https://img.shields.io/discord/532365473602600965)
![Twitter handle](https://img.shields.io/twitter/follow/rive_app.svg?style=social&label=Follow)

# rive-rs

![Rive hero image](https://cdn.rive.app/rive_logo_dark_bg.png)

A Rust runtime library for [Rive](https://rive.app).

## Table of contents

- ⭐️ [Rive Overview](#rive-overview)
- 🚀 [Getting Started](#getting-started)
- 👨‍💻 [Contributing](#contributing)
- ❓ [Issues](#issues)

## Rive Overview

[Rive](https://rive.app) is a real-time interactive design and animation tool that helps teams
create and run interactive animations anywhere. Designers and developers use our collaborative
editor to create motion graphics that respond to different states and user inputs. Our lightweight
open-source runtime libraries allow them to load their animations into apps, games, and websites.

🏡 [Homepage](https://rive.app/)

📘 [General help docs](https://help.rive.app/)

🛠 [Learning Rive](https://rive.app/learn-rive/)

## Getting Started

You will need a Rust toolchain and a C compiler to build. You can can install
the Rust toolchain using [rustup].

To open the included viewer, run:

```bash
$ cargo run --release
```

Then, drop any `.riv` file into the window to open it. Scroll to control the size of
the grid of copies.

[rustup]: https://rustup.rs

### Awesome Rive

For even more examples and resources on using Rive at runtime or in other tools, checkout the [awesome-rive](https://github.com/rive-app/awesome-rive) repo.

## Contributing

We love contributions! Check out our [contributing docs](./CONTRIBUTING.md) to get more details into
how to run this project, the examples, and more all locally.

## Issues

Have an issue with using the runtime, or want to suggest a feature/API to help make your development
life better? Log an issue in our [issues](https://github.com/rive-app/rive-rs/issues) tab! You
can also browse older issues and discussion threads there to see solutions that may have worked for
common problems.

### Known Issues

The current [Vello] render back-end does not render image meshes correctly and may start
rendering incorrectly when rendering a very large number of animations.

[Vello]: https://github.com/linebender/vello

