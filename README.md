# dfits-rs
A rust rewrite of dfits

# Motivation
`dfits` (and it's sister program `fitsort`) are amazingly useful pieces of software for batch processing the headers of many fits files. These were originally built in C by Nicolas Devillard (The original eclipse packages can be found on hit [github page](https://github.com/ndevilla/eclipse)). 

However, the main webpage for these tools is beginning to go [stale](https://www.eso.org/sci/software/eclipse/eug/eug/node8.html) with some links no longer working. The fear of loosing these tools is too great. Therefore some effort has gone in to preserve them in some way. Noteably [storing the raw c files](https://github.com/granttremblay/eso_fits_tools) and a [python rewrite](https://github.com/Romain-Thomas-Shef/dfitspy).

This repo, along with [fitsort-rs](https://github.com/TrystanScottLambert/fitsort-rs) are rust rewrites of both `dfits` and `fitsort` which aims to 1) preserve the functionality of the original. And 2) allow improvements if needed. This current version aims to be a truthful remake of the original matching the output exactly but I don't guarantee this will always be the case (But version 0.1.0 will be).

# Usage
`dfits` is already useful as is, on both a single fits file, or on multiple. 

```bash
dfits example.fits
```

will print out the header of `example.fits`. This is preferred to other methods like using `more` which includes some foreign characters. What makes `difts` particularly powerful is its combination with other util-tools like `grep`. For example, to only look at the EXPTIME of fits file, we can simply pipe the input into `grep`

```bash
dfits example.fits | grep EXPTIME
```

Wildcard characters can also be used seamlessly, say to get the EXPTIME of all fits files
```bash
dfits *.fits | grep EXPTIME
```

Even better is to make use of [fitsort](https://github.com/TrystanScottLambert/fitsort-rs) to parse the input from `dfits` (see the documentation for `fitsort`). 

```bash
dfits i_*.fits | fitsort PC1_1 PC2_2
```

Sometimes the main information you are looking for is in another file extension. The `-x` flag can be used to specify a specific extension with `0` meaning all extensions, and `N` being the Nth extension.

```bash
dfits example.fits # Defaults to the main extension (0)
dfits -x 0 example.fits # The main extension as well as any others
dfits -x 2 *.fits # The 2nd extension 
```

# Install
We provide several easy options for installing `dfits`.
## Downloading binaries
### Macos

For newer macs (m-series) the following commands should work.
```
curl -L -o dfits https://github.com/trystanscottlambert/dfits-rs/releases/download/v0.1.0/dfits-aarch64-apple-darwin
chmod +x dfits-aarch64-apple-darwin
sudo mv dfits-aarch64-apple-darwin /usr/local/bin/dfits
```
Alternatively for older macs
```
curl -L -o dfits https://github.com/trystanscottlambert/dfits/releases/download/v0.4.2/dfits-x86_64-apple-darwin
chmod +x dfits-x86_64-apple-darwin
sudo mv dfits-x86_64-apple-darwin /usr/local/bin/dfits
```

### Linux
For Ubuntu/Debian
```
curl -L -o dfits https://github.com/trystanscottlambert/dfits/releases/download/v0.4.2/dfits-x86_64-unknown-linux-gnu
chmod +x dfits-x86_64-unknown-linux-gnu
sudo mv dfits-x86_64-unknown-linux-gnu /usr/local/bin/dfits
```

## Building from source
If you don't want to download binaries and prefer to compile everything from source, then this can be done very easily using rust.

If you don't have rust first install it with the following command and follow the prompts.
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Clone the repo and `cd` into it and build the binaries.
```
git clone git@github.com:TrystanScottLambert/dfits-rs.git
cd dfits-rs
cargo build --release
```

Move the binary into /usr/local/bin
```
sudo mv targets/release/dfits /usr/local/bin/
```

## Cargo

If you are already comfortable with rust then you can just install `dfits` using cargo. 

```
cargo install dfits-rs
``` 


