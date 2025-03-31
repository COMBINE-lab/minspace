# minspace


## usage

The progam takes as input a FASTA format file, and converts the first contained record into 
(canonical) minimizer space using random minimizers and the provided parameters.

The output is a binary file in the following format:

The output contains 2 64-bit integers (`u64`) followed by an array of a type specified based
on these integers' values.
  * The first u64 integer `N` is the length of the output
  * The second u64 `M` is the maximum character (integer value) in the output
  * If either `N` or `M` is >= `i32::MAX`, then the following array is an array of `N` `u64` integers,
  otherwise it is an array of `N` `u32` integers.


```{bash}
Usage: minspace [OPTIONS] --input <INPUT> --output <OUTPUT>

Options:
  -i, --input <INPUT>    input file
  -o, --output <OUTPUT>  output file
  -w, --w <W>            window length [default: 31]
  -l, --l <L>            minimizer length [default: 10]
  -h, --help             Print help
  -V, --version          Print version
```
