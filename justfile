ci:
    cargo build
    cargo build --examples
    cargo clippy -- -D warnings
    cargo clippy --examples -- -D warnings
    cargo test

push flags='':
    git push -u github $(git branch --show-current) {{flags}}
    git push -u origin $(git branch --show-current) {{flags}}
