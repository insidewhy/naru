# Usage

```bash
chosen=$(ls | sdr)
echo the user chose $chosen
```

To achieve this `/dev/tty` is used for user interaction and `stdout` for the result.
 
# TODO

* configuration file, currently you use `j`/`n` to move down and `e`/`k` to move up.
* fuzzy selection.
