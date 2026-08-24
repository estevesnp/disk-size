# disk-size

small utility to print out the real disk size of a directory

## usage

```
usage: disk-size [dir] [-h] [-b] [-H]

flags:
  -h, --help                print this message
  -t, --time                print time it took to calculate size
  -b, --bytes               print size in bytes
  -H, --human-readable      print size in human readable format (default)
```

## build

`cargo build --release`
