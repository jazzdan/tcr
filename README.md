# TCR Tool

## Install

Current instructions for building: 

1. Install Rust
2. Clone the Repo `git clone https://github.com/jazzdan/tcr`
3. Cd into the repo and run `cargo build`
4. The TCR binary should be installed to `/target/debug/tcr`

## Usage

The `tcr` binary should be aliased or added to your $PATH variable to make it accessible from different directories. 

To use the tool, run `tcr` on a local Git repo of your choice. The tool expects a `.tcr` file to be present in the directory where it's run. A `.tcr` file has the following structure:

```
{
    "build_cmd": "cargo build",
    "test_cmd": "cargo test",
    "revert_cmd": "git reset HEAD --hard",
    "commit_cmd": "git commit -am working"
}
```

The keys in the `.tcr` file must be literals per the example, but the command values should be modified to meet your needs. For example, the above is for a Rust project -- hence the `cargo` commands.
