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
height = 20

[bindings]
c-n = "select-next"
c-e = "select-prev"
c-j = "select-next"
c-k = "select-prev"
```

## Using with neovim-fuzzy

```vim
let g:fuzzy_executable = 'naru'
```
 
# Status

`naru` is still in development, you will not want to use it yet.
