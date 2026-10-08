# joeanselpuplava.github.io

My personal portfolio website, built to look and feel like a terminal app.

**Live site:** https://joeanselpuplava.github.io/

It's written in Rust with [Ratatui](https://ratatui.rs), compiled to WebAssembly, and drawn in the browser with [ratzilla](https://github.com/orhun/ratzilla). [Trunk](https://trunkrs.dev) builds it.

## Features

- **Vim-style navigation:** `j`/`k`, `gg`/`G`, counts like `5j`, and `Ctrl-d`/`Ctrl-u`
- **Nested menus:** `Space`/`Enter` opens a directory and `Backspace` goes back. Directories are marked with `/`.
- **Mouse support:** click to focus a pane or pick a menu item, double-click to open a directory, scroll with the wheel, and click the breadcrumb to go back home
- **Remembered scroll position:** each page keeps its own scroll position
- **Word wrapping:** bullet points wrap with hanging indents
- **Clickable links** for email, LinkedIn, and GitHub
- **Hint bar:** shows the keys that work right now
- **Light and dark themes:** the light theme's text meets WCAG AAA contrast
- **Animated rain background** made with [tui-rain](https://github.com/SuperJappie08/tui-rain)

## Controls

| Key | Action |
|---|---|
| `j` / `k` (or `↓` / `↑`) | Move down / up in the menu, or scroll the content |
| `gg` / `G` | Jump to the top / bottom |
| `Ctrl-d` / `Ctrl-u` | Scroll half a page down / up |
| `h` / `l` (or `←` / `→`) | Focus the menu / content pane |
| `Space` / `Enter` | Open a directory |
| `Backspace` | Go back to the parent menu |
| `i` | Switch between light and dark themes |
| `x` | Turn the rain on or off |
| `<number>` + motion | Repeat a motion, e.g. `5j` |

## Running locally

You need Rust, the WebAssembly target, and Trunk:

```sh
rustup target add wasm32-unknown-unknown
cargo install --locked trunk
```

Then start the dev server:

```sh
trunk serve
```

Open http://127.0.0.1:8080. The page rebuilds and reloads when you save a file.

To make the same optimized build that gets deployed:

```sh
trunk build --release   # output goes to dist/
```

## Project structure

```
index.html            Trunk entry point (page styles, font)
src/
  main.rs             Sets up the terminal and runs the draw loop
  app.rs              App state: menus, focus, selection, scroll
  event.rs            Keyboard and mouse listeners on the page
  update.rs           Turns keys and clicks into state changes
  home.rs             Layout and drawing: menu, content, hint bar
  wrap.rs             Word wrapping with hanging indents
  theme.rs            Light and dark color themes
  rain.rs             Background rain animation
  content/            The site's pages
    mod.rs            Top-level menu and helpers (heading, bullet, link, ...)
    about.rs
    experience.rs
    contact.rs
    projects/         One file per project
```

## Adding content

Each page is a Rust file with a `text()` function that returns its contents. Each directory is a folder with a `mod.rs` that lists the pages in it.

To add a page:

1. Create `src/content/<name>.rs`. Copy an existing page as a starting point.
2. Add `mod <name>;` to `src/content/mod.rs`.
3. Add `Entry::page("Title", <name>::text)` to `MENU`.

To add a project, do the same in `src/content/projects/` and add it to `ENTRIES` there.

Pages are built from the helpers in `src/content/mod.rs`, so they all look the same:

```rust
heading("Title")                          // page title
meta("Role | Dates")                      // italic details
section("Design")                         // section heading
bullet("Some text")                       // bullet point
labeled_bullet("Label:", "Some text")     // bullet with a bold label
link("github.com/me", "https://...")      // clickable link
```

## Deployment

Every push to `main` runs [`.github/workflows/deploy.yml`](.github/workflows/deploy.yml). It builds the site with `trunk build --release` and publishes `dist/` to GitHub Pages. In the repo's Pages settings, the source must be set to **GitHub Actions**.
