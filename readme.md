# Usage

```bash
chosen=$(ls | naru)
echo the user chose $chosen
```

To achieve this `/dev/tty` is used for user interaction and `stdout` for the result.

`naru` shows its results directly below the cursor position (scrolling the screen upward if necessary). This allows it to be used with editor plugins.

`naru` keybindings are configurable and it has a unique feature: it allows you to select multiple matches.

## Configuration file

The configuration file uses the `toml` format, here is an example showing the defaults:


```toml
[window]
height = 0

[bindings]
c-j = "select-next"
c-k = "select-prev"
```

For `window.height`, positive numbers specify the height in lines, 0 means "full height" and the negative number `-n` means `full_height - n`.

## Using with neovim-fuzzy

```vim
let g:fuzzy_executable = 'naru'
```
