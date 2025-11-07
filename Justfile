fmt:
    dx fmt && cargo sort && cargo fmt --all

css:
    deno task css

css-watch:
    deno task css:watch

dx-serve:
    dx serve

css-minify:
    deno task css:minify

[parallel]
serve: css-watch dx-serve

desktop-icon:
    cargo tauri icon assets/logo.png -o icons && cd icons && rm -rf android && rm -rf ios
