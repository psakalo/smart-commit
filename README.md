# Smart Commit

Smart Commit is a command-line tool designed to enhance your Git workflow by leveraging the power of OpenAI's GPT models. It automatically generates commit messages based on the changes made in your Git repository.

## Installation

**Using homebrew:**
```shell
brew tap psakalo/homebrew-tap
brew install smart-commit
```

## Configuration

`smart-commit` required OpenAI API key to run. 

1. If you don't have an OpenAI account, create one at https://openai.com/
2. Generate API key at https://platform.openai.com/api-keys
3. Key can be passed into `smart-commit` via `--openai-api-key` argument or `OPENAI_API_KEY` environment variable


## How to use

Idea is to keep `smart-commit` as simple as possible. App will output one line with commit message to `stdout`. This allows it to be chained with other apps, specifically with `git commit` command.

- Use as a standalone application:
```shell
$ 
```


#### Default
```shell
$ smart-commit
```

