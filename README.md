# tranclator
This is basically just a fancy find and replace in the terminal. I made it for
[this](https://www.youtube.com/watch?v=aE-mZH4uJX8&t=1547s). The default config for "neuu english" can be
found in [`langs/tranclator.toml`](langs/tranclator.toml). The schema can be found [`here`](tranclator-schema.json)

## Why?
Because it's fun!

## Building
### Prerequisites
- [`git`](https://git-scm.com/)
- [`rust`](https://www.rust-lang.org/learn/get-started)

### Cloning
First clone the repository via
```bash
$ git clone https://github.com/Sietse2202/tranclator.git
```

### Build
To then build the program
```bash
$ cargo build --release
```
the executable for your platform is now placed in `target/`

## License
The Unlicense, I don't care about this code, do with it whatever you want! 
