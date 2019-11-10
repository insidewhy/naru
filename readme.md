# Usage

```bash
chosen=$(ls | toss)
echo the user chose $chosen
```

To achieve this `/dev/tty` is used for user interaction and `stdout` for the result.

`toss` shows its results directly below the cursor position (scrolling the screen upward if necessary). This allows it to be used with editor plugins.

`toss` keybindings are configurable and it has a unique feature: it allows you to select multiple matches.

## Using with neovim-fuzzy

```vim
let g:fuzzy_executable = 'toss'
```
 
# Status

`toss` is still in development, you will not want to use it yet.
