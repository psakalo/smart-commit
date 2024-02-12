# Smart Commit

Smart Commit is a command-line tool designed to enhance your Git workflow by leveraging the power of OpenAI's GPT models. It automatically generates commit messages based on the changes made in your Git repository.

## Installation

**Using homebrew:**
```shell
brew install psakalo/homebrew-tap/smart-commit
```

## Configuration

To use `smart-commit`, an OpenAI API key is required. Follow these steps to set up:

1. Create an OpenAI account at [openai.com](https://openai.com/) if you haven't already.
2. Generate API key at https://platform.openai.com/api-keys
4. `smart-commit` securely stores the API key using the [keyring](https://github.com/hwchen/keyring-rs) crate, ensuring your key is safely managed.

Additionally, you can specify the API key directly:

- Pass the API key to `smart-commit` using the `--openai-api-key` argument or the `OPENAI_API_KEY` environment variable.
- These methods have higher priority but do not overwrite the value stored by keyring.

## How to use

Smart Commit is designed to be flexible, providing you with a streamlined approach to generating commit messages. Unlike other tools that serve as wrappers for git commit or are solely intended for use with prepare-commit-msg, Smart Commit outputs a single commit message to stdout. This design gives you the freedom to decide how to utilize the generated message.

**Interactive Mode:**
```shell
smart-commit | git commit -F - 
```

In this mode, you will be presented with up to 3 commit message options (adjustable via the `-r <number>` argument), allowing you to select the most suitable message for your commit.
You can choose to include `-e` option with `git commit` to edit commit message.
